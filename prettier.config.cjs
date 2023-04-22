module.exports = {
  endOfLine: "lf",
  printWidth: 100,
  proseWrap: "always",
  overrides: [
    {
      files: "*.yaml",
      options: {
        proseWrap: "preserve",
      },
    },
  ],
};
