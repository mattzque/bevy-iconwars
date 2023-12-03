const { promises: fs } = require('fs');
const path = require('path');
const render = require('svg-render');
const sharp = require('sharp');
const solidIcons = require('@fortawesome/free-solid-svg-icons');
const brandIcons = require('@fortawesome/free-brands-svg-icons');

function getOnlyIcons(icons) {
    return Object.values(icons).filter((icon) => !!icon.iconName);
}

const icons = getOnlyIcons({ ...solidIcons, ...brandIcons });

function getIconSvg(icon) {
    const [width, height, _a, _b, path] = icon.icon;
    return `
        <?xml version="1.0" encoding="UTF-8" standalone="no"?>
        <svg 
          width="${width}"
          height="${height}"
          viewBox="0 0 ${width} ${height}"
          xmlns="http://www.w3.org/2000/svg"
          xmlns:xlink="http://www.w3.org/1999/xlink">
            <path d="${path}" fill="white" />
        </svg>
    `;
}

const SIZE = 64;

const promises = icons.map(async (icon) => {
    const buffer = Buffer.from(getIconSvg(icon));
    const png = await render({
        buffer,
        width: SIZE,
        height: SIZE
    });
    const filename = path.join('output', 'icons', `${icon.iconName}.png`);
    await fs.writeFile(filename, png);
    return filename;
});

const TEXTURE_SIZE = 2048;
const TEXTURE_TILES = TEXTURE_SIZE / SIZE;

async function createAndSaveSpriteSheet(icons, filename) {
    let image = sharp({
        create: {
            width: TEXTURE_SIZE,
            height: TEXTURE_SIZE,
            channels: 4,
            background: { r: 0, g: 0, b: 0, alpha: 0 }
        }
    });

    // Composite images onto the sprite sheet
    const tiles = [];
    const composites = [];
    icons.forEach((filename, index) => {
        const row = Math.floor(index / TEXTURE_TILES);
        const col = index % TEXTURE_TILES;
        composites.push({
            input: filename,
            top: row * SIZE,
            left: col * SIZE
        });
        tiles.push({
            name: path.basename(filename, path.extname(filename)),
            x: col * SIZE,
            y: row * SIZE
        });
    });

    await image.composite(composites)
        .png()
        .toFile(filename);
    console.log('Written icon textures:', filename);

    return tiles;
}

(async () => {
    const filenames = [...new Set(await Promise.all(promises))];
    console.log('Rendered', filenames.length, 'icons');
    const tiles = TEXTURE_TILES * TEXTURE_TILES;
    const textures = Math.ceil(filenames.length / tiles);
    const metas = [];
    for (let i = 0; i < textures; i++) {
        let files_i = i * tiles;
        let files = filenames.slice(files_i, files_i + tiles);
        const filename = path.join('output', `icons_${i}.png`);
        let meta = await createAndSaveSpriteSheet(files, filename);
        metas.push({ filename: path.basename(filename), tiles: meta, tileWidth: SIZE, tileHeight: SIZE });
    }
    await fs.writeFile(path.join('output', `icons.icon.json`), JSON.stringify(metas));
})();