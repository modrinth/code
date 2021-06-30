use futures::{executor, future};
use std::path::Path;

#[tokio::main]
async fn main() {
    let mut thing1 = theseus::launcher::meta::fetch_version_manifest()
        .await
        .unwrap();

    // future::join_all(thing1.versions.iter().map(|x| async move {
    //     println!("{}", x.url);
    //     let version = theseus::launcher::meta::fetch_version_info(x)
    //         .await
    //         .unwrap();
    //
    //     println!("{:?}", version);
    // }))
    // .await;

    if let Some(version) = thing1.versions.iter().find(|x| &*x.id == "1.17") {
        println!("{}", version.id);
        let thing = theseus::launcher::meta::fetch_version_info(&version)
            .await
            .unwrap();

        theseus::launcher::download::download_client(&Path::new("./versions"), &thing).await;
    }
}
