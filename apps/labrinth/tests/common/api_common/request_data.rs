// The structures for project/version creation.
// These are created differently, but are essentially the same between versions.

use labrinth::util::actix::MultipartSegment;

use crate::common::dummy_data::TestFile;

#[allow(dead_code)]
pub struct ProjectCreationRequestData {
    pub slug: String,
    pub jar: Option<TestFile>,
    pub segment_data: Vec<MultipartSegment>,
}

#[allow(dead_code)]
pub struct VersionCreationRequestData {
    pub version: String,
    pub jar: Option<TestFile>,
    pub segment_data: Vec<MultipartSegment>,
}

#[allow(dead_code)]
pub struct ImageData {
    pub filename: String,
    pub extension: String,
    pub icon: Vec<u8>,
}
