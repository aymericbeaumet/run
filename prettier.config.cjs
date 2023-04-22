module.exports = {
  endOfLine: "lf",
  printWidth: 100,
  proseWrap: "preserve",
  overrides: [
    {
      files: "*.md",
      options: {
        proseWrap: "always",
      },
    },
  ],
};
