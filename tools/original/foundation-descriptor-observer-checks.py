#!/usr/bin/env python3
"""Read-only probe contracts; native seed vectors are test evidence only."""
import importlib.util
import json
import os
from pathlib import Path
import struct
import unittest

spec=importlib.util.spec_from_file_location('descriptor',Path(__file__).with_name('foundation-descriptor-observer.py'))
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)


class DescriptorProbeContracts(unittest.TestCase):
    def test_only_nonzero_mathematical_triples_match(self):
        block=struct.pack('<12I',0,0,0,1640814399,2786966973,1121108495,123,456,789,1,2,3)
        rows,_=module.scan_block(block)
        self.assertEqual([row[1] for row in rows],[(1640814399,2786966973,1121108495)])

    def test_cross_chunk_triple_survives_and_completed_tail_is_ignored(self):
        block=struct.pack('<III',1640814399,2786966973,1121108495)
        rows,_=module.scan_block(block,8)
        self.assertEqual(len(rows),1)
        self.assertEqual(module.scan_block(block,12)[0],[])

    def test_public_tuple_scan_covers_other_streams_and_alternative60(self):
        block=struct.pack('<8I',12,2,15,17,42,9,5,1)
        _,states=module.scan_block(block)
        self.assertIn((0,15,12),states)
        self.assertIn((16,81,42),states)

    def test_observed_native_chain_matches_stage_without_exporting_location(self):
        triple=(1640814399,2786966973,1121108495)
        descriptors={triple:{'occurrences':2,'mappingKinds':{'[heap]'},'smallNeighborValues':{0,84}}}
        result=module.summarize(descriptors,{35:{3199781928}},
            {'spawnSeed':2786966973,'rooms':8},module.stage_positions(2478271409))
        self.assertEqual(result['seedChainCandidates'][0]['stage35Positions'],[45])
        self.assertTrue(result['seedChainCandidates'][0]['matchesActualCurrentSpawn'])
        self.assertFalse(result['validation']['candidateCountEqualsLogRoomCount'])
        self.assertNotIn('address',json.dumps(result))
        self.assertFalse(result['validation']['fullInternalState'])

    def test_empty_candidates_are_explicitly_incomplete(self):
        result=module.summarize({}, {}, {'spawnSeed':1,'rooms':8}, {})
        self.assertEqual(result['validation']['currentSpawnCandidates'],0)
        self.assertFalse(result['validation']['candidateCountEqualsLogRoomCount'])

    def test_nonchild_target_is_rejected_before_memory_reads(self):
        result=module.observe(os.getpid(),{'gateBlocked':True,'label':'guard-check'})
        self.assertEqual(result['status'],'fail')
        self.assertEqual(result['bytesRead'],0)
        self.assertFalse(result['memoryWritten'])
        self.assertFalse(result['executableMappingsRead'])


if __name__=='__main__':unittest.main()
