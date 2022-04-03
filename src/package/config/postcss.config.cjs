const config = {
    plugins: [
        require('postcss-import-ext-glob'),
        require('postcss-import'),
        require('postcss-strip-inline-comments'),
        require('postcss-extend-rule'),
        require('postcss-nested'),
        require('postcss-preset-env')({
            features: {
                'custom-media-queries': {
                    importFrom: [
                        {
                            customMedia: {
                                '--sm': '(min-width: 544px)',
                                '--md': '(min-width: 768px)',
                                '--lg': '(min-width: 1012px)',
                                '--xl': '(min-width: 1280px)',
                            },
                        },
                    ]
                }
            }
        }),
        require('postcss-pxtorem'),
        require('autoprefixer'),
        process.env.NODE_ENV === 'development' && require('cssnano')({
            preset: 'default',
        })
    ],
};

module.exports = config;
