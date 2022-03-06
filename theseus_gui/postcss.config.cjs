const dev = process.env.NODE_ENV === 'development';

const config = {
    plugins: [
        require('postcss-import')(),
        require('autoprefixer')(),
        require('postcss-nested')(),
        require('postcss-extend-rule')(),
        //require('postcss-preset-env')(), Errors with cssnano
        require('postcss-media-minmax')(),

        !dev &&
        require('cssnano')({
            preset: 'default',
        }),
    ],
};

module.exports = config;
