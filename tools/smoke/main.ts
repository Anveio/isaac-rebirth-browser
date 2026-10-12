import { Application, Assets, Sprite } from 'pixi.js';
import spriteUrl from '../../references/isaac/rebirth/resources/gfx/characters/costumes/character_001_isaac.png?url';

async function check() {
  const app = new Application();
  await app.init({ width: 640, height: 360, preference: 'webgl', background: '#30272a' });
  document.querySelector('#render')!.appendChild(app.canvas);
  const texture = await Assets.load(spriteUrl);
  const sprite = new Sprite(texture);
  sprite.texture.source.scaleMode = 'nearest';
  sprite.scale.set(2);
  sprite.position.set(24, 24);
  app.stage.addChild(sprite);
  const health = {
    renderer: app.renderer.name,
    originalSprite: { width: texture.width, height: texture.height },
    webAssembly: WebAssembly.validate(new Uint8Array([0,97,115,109,1,0,0,0])),
    webAudio: typeof AudioContext === 'function',
    indexedDB: typeof indexedDB === 'object',
  };
  document.querySelector('#health')!.textContent = JSON.stringify(health, null, 2);
  document.body.dataset.ready = 'true';
}

check().catch(error => {
  document.querySelector('#health')!.textContent = String(error);
  document.body.dataset.ready = 'failed';
});
