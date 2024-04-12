Generate JSX from SVG File

```bash
npx @svgr/cli --icon 16px --ext jsx -d ./jsx ./svg/Add.svg
```

执行的时候要注释掉 `prettier.config.cjs` 里面的

```javascript
require.resolve("prettier-plugin-organize-imports"),
require.resolve("prettier-plugin-tailwindcss"),
```

不然会报错。还没花时间排查。
