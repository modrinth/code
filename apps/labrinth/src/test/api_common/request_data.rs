// The structures for project/version creation.
// These are created differently, but are essentially the same between versions.

use crate::util::actix::MultipartSegment;

use crate::test::dummy_data::TestFile;

pub struct ProjectCreationRequestData {
    pub slug: String,
    pub jar: Option<TestFile>,
    pub segment_data: Vec<MultipartSegment>,
}

pub struct VersionCreationRequestData {
    pub version: String,
    pub jar: Option<TestFile>,
    pub segment_data: Vec<MultipartSegment>,
}

pub struct ImageData {
    pub filename: String,
    pub extension: String,
    pub icon: Vec<u8>,
}
