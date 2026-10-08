use toasty::Embed;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub enum FabriclikeLoader {
    Fabric,
    Quilt,
}
