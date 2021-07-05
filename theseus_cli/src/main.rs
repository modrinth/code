use futures::{executor, future};
use std::path::Path;
use theseus::launcher::launch_minecraft;
use theseus::launcher::meta::ArgumentType;

#[tokio::main]
async fn main() {
    launch_minecraft("1.15.2", &Path::new("./test")).await;

    // let mut thing1 = theseus::launcher::meta::fetch_version_manifest()
    //     .await
    //     .unwrap();
    //
    // future::join_all(thing1.versions.iter().map(|x| async move {
    //     //println!("{}", x.url);
    //     let version = theseus::launcher::meta::fetch_version_info(x)
    //         .await
    //         .unwrap();
    //
    //     if let Some(args) = &version.minecraft_arguments {
    //         println!("{:?}", args);
    //     }
    //     if let Some(args) = &version.arguments {
    //         println!("{:?}", args.get(&ArgumentType::Game).unwrap());
    //     }
    // }))
    // .await;
}
