// Declared public startup IO, selected before case generation or either game.
// Never infer this version from an observed storage key or a missing field.
import assert from 'node:assert/strict';

export function optionsPreferenceContract(preferencesSchema) {
  assert.ok(preferencesSchema === 2 || preferencesSchema === 3,
    'Options preference schema must be explicitly 2 or 3');
  const initial = {schema: preferencesSchema, sfxVolume: '1.0000', musicVolume: '1.0000', fullscreen: false};
  const saved = {schema: preferencesSchema, sfxVolume: '1.0000', musicVolume: '0.9000', fullscreen: false};
  if (preferencesSchema === 3) { initial.mapOpacity = '0.3000'; saved.mapOpacity = '0.3000'; }
  return {schema: 1, preferencesSchema, storageKey: `isaac-rebirth.options.v${preferencesSchema}`, initial, saved};
}
