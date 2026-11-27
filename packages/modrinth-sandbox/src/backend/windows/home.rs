use std::{
    fs::{self, OpenOptions},
    io::ErrorKind,
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt, io::AsRawHandle},
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use uuid::Uuid;
use windows::Win32::{
    Foundation::{GENERIC_WRITE, HANDLE},
    Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    },
    System::{
        IO::DeviceIoControl, Ioctl::FSCTL_SET_REPARSE_POINT,
        SystemServices::IO_REPARSE_TAG_MOUNT_POINT,
    },
};

use super::appcontainer::strip_verbatim_prefix;

/// Makes a folder inside of `real_home` available at the same location inside of
/// `sandbox_home`, so that programs can find it relative to their home directory.
pub fn link_into_sandbox_home(
    folder: &Path,
    real_home: &Path,
    sandbox_home: &Path,
) -> Result<()> {
    let Some(relative) = relative_to(folder, real_home) else {
        return Ok(());
    };
    let Some(name) = relative.file_name() else {
        return Ok(());
    };
    // Programs find this using environment variables, which we override
    if relative
        .iter()
        .next()
        .is_some_and(|first| first.eq_ignore_ascii_case("AppData"))
    {
        return Ok(());
    }
    let Ok(target) = folder.canonicalize() else {
        return Ok(());
    };
    // Linking a folder which contains the sandbox would create a loop
    if !target.is_dir() || sandbox_home.canonicalize()?.starts_with(&target) {
        return Ok(());
    }

    let link = sandbox_home.join(&relative);
    if link.canonicalize().is_ok_and(|path| path == target) {
        return Ok(());
    }

    // Replace links left by an earlier launch, otherwise we would be modifying
    // the real folder that they point to
    let mut directory = sandbox_home.to_path_buf();
    for component in relative.parent().into_iter().flat_map(Path::components) {
        directory.push(component);
        remove_link(&directory)?;
        if let Err(err) = fs::create_dir(&directory)
            && err.kind() != ErrorKind::AlreadyExists
        {
            return Err(err).with_context(|| format!("creating {directory:?}"));
        }
    }

    remove_link(&link)?;
    match fs::remove_dir(&link) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        // Keep anything that was saved before the folder was granted
        Err(_) => {
            let mut backup_name = name.to_owned();
            backup_name.push(format!("-backup-{}", Uuid::now_v7()));
            fs::rename(&link, link.with_file_name(backup_name))
                .with_context(|| format!("backing up {link:?}"))?;
        }
    }

    create_junction(&link, &strip_verbatim_prefix(&target)).inspect_err(|_| {
        _ = fs::remove_dir(&link);
    })
}

/// Doesn't affect the target of the link.
fn remove_link(path: &Path) -> Result<()> {
    if path
        .symlink_metadata()
        .is_ok_and(|metadata| metadata.is_symlink())
    {
        fs::remove_dir(path)
            .with_context(|| format!("removing link {path:?}"))?;
    }
    Ok(())
}

fn relative_to(path: &Path, base: &Path) -> Option<PathBuf> {
    let path = strip_verbatim_prefix(path);
    let base = strip_verbatim_prefix(base);

    let mut components = path.components();
    for base_component in base.components() {
        let component = components.next()?;
        if component.as_os_str().to_string_lossy().to_lowercase()
            != base_component.as_os_str().to_string_lossy().to_lowercase()
        {
            return None;
        }
    }

    let relative = components.as_path();
    relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
        .then(|| relative.to_path_buf())
}

fn create_junction(link: &Path, target: &Path) -> Result<()> {
    let target_str = target.as_os_str().encode_wide().collect::<Vec<_>>();
    ensure!(
        target_str.get(1) == Some(&u16::from(b':')),
        "junctions can't point to network path {target:?}"
    );

    let substitute_name = r"\??\"
        .encode_utf16()
        .chain(target_str.iter().copied())
        .collect::<Vec<_>>();
    let substitute_name_size = size_of_val(substitute_name.as_slice());
    let print_name_size = size_of_val(target_str.as_slice());

    let mut data = vec![
        0,
        u16::try_from(substitute_name_size)?,
        (substitute_name_size + size_of::<u16>()).try_into()?,
        print_name_size.try_into()?,
    ];
    data.extend(substitute_name);
    data.push(0);
    data.extend(target_str);
    data.push(0);

    let mut buffer = Vec::<u8>::new();
    buffer.extend(IO_REPARSE_TAG_MOUNT_POINT.to_le_bytes());
    buffer.extend(u16::try_from(size_of_val(data.as_slice()))?.to_le_bytes());
    buffer.extend(0_u16.to_le_bytes());
    buffer.extend(data.into_iter().flat_map(u16::to_le_bytes));

    fs::create_dir(link).with_context(|| format!("creating {link:?}"))?;
    let directory = OpenOptions::new()
        .access_mode(GENERIC_WRITE.0)
        .custom_flags(
            (FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT).0,
        )
        .open(link)
        .with_context(|| format!("opening {link:?}"))?;

    let mut returned = 0;
    unsafe {
        DeviceIoControl(
            HANDLE(directory.as_raw_handle()),
            FSCTL_SET_REPARSE_POINT,
            Some(buffer.as_ptr().cast()),
            buffer.len() as u32,
            None,
            0,
            Some(&raw mut returned),
            None,
        )
    }
    .with_context(|| format!("linking {link:?} to {target:?}"))
}
