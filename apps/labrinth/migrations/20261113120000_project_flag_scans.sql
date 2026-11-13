CREATE TABLE project_flag_image_scans (
    image_url TEXT NOT NULL,
    generation BIGINT NOT NULL,
    ai_generated DOUBLE PRECISION,
    explicit DOUBLE PRECISION,
    PRIMARY KEY (image_url, generation)
);
