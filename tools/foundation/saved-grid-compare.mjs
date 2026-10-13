// Source observations are comparisons only; this module never supplies runtime inputs.
import assert from 'node:assert/strict';

export function compareSavedRoomGrid(summary, grid, observed, { requireRecords = false } = {}) {
  assert.equal(summary.currentRoomRecord, observed.savedRun.currentRoomRecord,
    'Actual current descriptor row identity must match source');
  assert.equal(observed.currentRoomSeedSamples?.length, 9,
    'Current Room.Save requires independently observed active-row samples');
  assert.equal(summary.currentRoomSeedSamples?.length, 9);
  const differences = [];
  for (const [index, source] of observed.currentRoomSeedSamples.entries()) {
    const sample = summary.currentRoomSeedSamples[index];
    assert.equal(sample.gridIndex, source.gridIndex);
    assert.deepEqual(sample.levelSeeds, sample.savedSeeds,
      'Production current row must retain actual Level→Saved copy ownership');
    assert.deepEqual(source.levelSeeds, source.savedSeeds,
      'Native current row must retain actual Level→Saved copy ownership');
    for (const field of ['levelSeeds', 'savedSeeds', 'occupancy']) {
      if (JSON.stringify(sample[field]) !== JSON.stringify(source[field])) {
        differences.push({ gridIndex: source.gridIndex, field,
          original: source[field], production: sample[field],
          sourceGridObject: source.object || null });
      }
    }
    if (requireRecords && source.occupancy !== 'padding') {
      assert.ok(source.levelRecord && source.savedRecord,
        'Grid-record acceptance requires newly observed actual Level/Saved records');
      assert.deepEqual(source.levelRecord, source.savedRecord,
        'Actual source Level/Saved current-cell record ownership must match');
      for (const field of ['kind', 'variant', 'state', 'sourceWord14', 'primarySeed', 'initialized', 'secondarySeed']) {
        if (sample.record?.[field] !== source.savedRecord[field]) {
          differences.push({ gridIndex: source.gridIndex, field: `record.${field}`,
            original: source.savedRecord[field], production: sample.record?.[field] ?? null,
            sourceGridObject: source.object || null });
        }
      }
    }
  }
  const emptyIndices = grid.cells.flatMap((cell, index) => cell.entity ? [] : [index]);
  const originalOccupiedProductionNull = emptyIndices.filter(index => !observed.liveGrid.emptyIndices.includes(index));
  const originalNullProductionOccupied = observed.liveGrid.emptyIndices.filter(index => !emptyIndices.includes(index));
  const dimensions = {
    original: [observed.liveGrid.width, observed.liveGrid.height, observed.liveGrid.width * observed.liveGrid.height],
    production: [grid.width, grid.height, grid.cells.length],
  };
  const dimensionsMatch = JSON.stringify(dimensions.original) === JSON.stringify(dimensions.production);
  const currentRoomSeedDiagnostic = {
    status: differences.length ? 'fail' : 'pass', roomRecord: summary.currentRoomRecord,
    comparedCells: 9, differences,
    reason: 'Paired current-row seeds and occupancy are required; source samples never enter production',
  };
  const gridOccupancyDiagnostic = {
    status: dimensionsMatch && JSON.stringify(emptyIndices) === JSON.stringify(observed.liveGrid.emptyIndices) ? 'pass' : 'fail',
    originalNullCells: observed.liveGrid.emptyIndices.length, productionNullCells: emptyIndices.length,
    originalOccupiedProductionNull, originalNullProductionOccupied, dimensions,
    reason: 'Actual whole live-grid occupancy is required; observed occupancy never changes production state',
  };
  const savedGridRequiredFailures = differences.map(difference => ({
    field: `currentRoomSeedSamples.${difference.field}`, gridIndex: difference.gridIndex,
    expected: difference.original, actual: difference.production,
  }));
  for (const [index, field] of ['room.grid.width', 'room.grid.height', 'room.grid.cells.length'].entries()) {
    if (dimensions.original[index] !== dimensions.production[index]) {
      savedGridRequiredFailures.push({ field, expected: dimensions.original[index], actual: dimensions.production[index] });
    }
  }
  if (JSON.stringify(emptyIndices) !== JSON.stringify(observed.liveGrid.emptyIndices)) {
    savedGridRequiredFailures.push({ field: 'liveGrid.emptyIndices',
      expected: observed.liveGrid.emptyIndices, actual: emptyIndices });
  }
  return { currentRoomSeedDiagnostic, gridOccupancyDiagnostic, savedGridRequiredFailures };
}
