const config = {
  plugins: [
    require('postcss-import'),
    require('postcss-strip-inline-comments'),
    require('postcss-extend-rule'),
    require('postcss-nested'),
    require('postcss-preset-env'),
    require('autoprefixer'),
    process.env.NODE_ENV === 'development' && require('cssnano')({
      preset: 'default',
    })
  ],
};

module.exports = config;
