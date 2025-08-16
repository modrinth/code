---
title: Portable Mode
description: Learn how to use Modrinth App in portable mode for maximum flexibility and convenience.
---

The Modrinth App supports **portable mode**, which allows you to run the application from any location without requiring installation to system directories. In portable mode, all application data is stored alongside the executable, making it perfect for USB drives, shared computers, or when you want a completely self-contained setup.

## What is Portable Mode?

Portable mode changes how the Modrinth App stores its data. Instead of using system directories like:

- `%APPDATA%\ModrinthApp` (Windows)
- `~/Library/Application Support/ModrinthApp` (macOS)
- `~/.local/share/ModrinthApp` (Linux)

All data is stored in a `ModrinthAppData` folder right next to the executable, making the entire installation truly portable.

## Benefits of Portable Mode

- **No Installation Required**: Run directly from any location
- **System Independence**: No registry entries or system directory modifications
- **Easy Backup**: Simply copy the entire folder to backup everything
- **Multi-System**: Use the same setup across different computers
- **Clean Removal**: Delete the folder to completely remove all traces
- **Isolation**: Perfect for testing without affecting main installations

## Enabling Portable Mode

There are two methods to enable portable mode:

### Method 1: portable.txt File (Recommended)

1. Create an empty file named `portable.txt` in the same directory as your Modrinth App executable
2. Launch the app normally
3. The app will automatically detect the file and enable portable mode

```
YourFolder/
├── Modrinth App.exe (or modrinth-app on Linux/macOS)
├── portable.txt          ← Create this file
└── ModrinthAppData/       ← Created automatically on first run
```

### Method 2: Environment Variable

Set the `MODRINTH_PORTABLE` environment variable to any value before launching the app:

```bash
# Windows (PowerShell)
$env:MODRINTH_PORTABLE="true"
.\ModrinthApp.exe

# Windows (Command Prompt)
set MODRINTH_PORTABLE=true
ModrinthApp.exe

# Linux/macOS
export MODRINTH_PORTABLE=true
./modrinth-app
```

## Directory Structure

When portable mode is enabled, your directory structure will look like this:

```
YourPortableFolder/
├── Modrinth App.exe          # The application executable
├── portable.txt              # Enables portable mode (Method 1)
└── ModrinthAppData/          # All app data stored here
    ├── profiles/             # Minecraft instances and profiles
    │   ├── vanilla/
    │   ├── modded/
    │   └── ...
    ├── caches/               # Downloaded files cache
    │   ├── mods/
    │   ├── resource_packs/
    │   └── ...
    ├── launcher_logs/        # Application logs
    ├── meta/                 # Metadata files
    └── app.db               # Application database
```

## Creating a Portable Installation

### From Source (Developers)

If you're building from source:

1. Build the application:

   ```bash
   pnpm app:build
   ```

2. Copy the executable from `target/release/` to your portable folder

3. Create the portable indicator:
   ```bash
   # Create portable.txt file
   echo "Portable mode enabled" > portable.txt
   ```

### From Release Binary

1. Download the Modrinth App executable from [modrinth.com/app](https://modrinth.com/app)
2. Place it in your desired portable folder
3. Create an empty `portable.txt` file next to the executable
4. Launch the app

## Use Cases

### USB Drive Setup

Perfect for carrying your complete Minecraft setup on a USB drive:

```
USB_Drive/
├── ModrinthApp/
│   ├── Modrinth App.exe
│   ├── portable.txt
│   └── ModrinthAppData/
└── other_files/
```

### Shared Computer

Use your personal setup on shared computers without affecting other users or requiring installation permissions.

### Testing Environment

Test different modrinth configurations or app versions without affecting your main installation.

### System Migration

Easily move your entire Minecraft setup to a new computer by copying the portable folder.

## Switching Between Modes

### Converting Regular Installation to Portable

1. Create a portable folder with the executable and `portable.txt`
2. Copy your existing data from the system directory to `ModrinthAppData/`
3. Launch the portable version

### Converting Portable to Regular Installation

1. Install the app normally
2. Copy data from `ModrinthAppData/` to the system directory
3. Remove the `portable.txt` file or unset the environment variable

## Technical Notes

- **Detection Priority**: Environment variable takes precedence over `portable.txt` file
- **Performance**: Portable mode has identical performance to regular installations
- **Platform Support**: Available on Windows, macOS, and Linux
- **First Launch**: The `ModrinthAppData` directory is created automatically on first run
- **File Permissions**: Ensure the portable folder has write permissions

## Troubleshooting

### App Not Detecting Portable Mode

- Verify `portable.txt` is in the same directory as the executable
- Check file permissions on the portable folder
- Try using the environment variable method instead

### Data Not Saving

- Ensure the portable folder has write permissions
- Check that antivirus software isn't blocking file creation
- Verify there's sufficient disk space

### Performance Issues

- Portable mode shouldn't affect performance
- If using a USB drive, ensure it has adequate read/write speeds
- Consider using USB 3.0+ for better performance

## Security Considerations

- **Portable Storage**: Be mindful of where you store portable installations
- **Data Encryption**: Consider encrypting USB drives with sensitive data
- **Access Control**: Set appropriate file permissions on shared systems
- **Backup**: Regularly backup your portable installation

Portable mode makes the Modrinth App incredibly flexible and convenient for various use cases while maintaining all the functionality of a standard installation.
