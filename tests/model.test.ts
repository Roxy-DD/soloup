import { describe, expect, it } from 'vitest';

import { achProgress } from '../apps/web/src/lib/model';
import type { AchStats } from '../apps/web/src/lib/types';

const stats: AchStats = {
  totalDays: 12,
  streak: 3,
  maxLitOneDay: 2,
  maxAttrLv: 0,
  maxSkillLv: 1.6,
  maxSkillCheckins: 7,
  projectsDone: 1,
  totalAttrs: 5,
};

describe('achievement progress', () => {
  it('uses the observable per-skill practice count for milestone progress', () => {
    expect(achProgress(stats, { stat: 'maxSkillCheckins', operator: '>=', value: 14 })).toEqual({
      text: '累计打卡 7 / 14 天',
      ratio: 0.5,
    });
  });

  it('returns no fabricated progress for unsupported statistics', () => {
    expect(achProgress(stats, { stat: 'unknownStat', operator: '>=', value: 4 })).toBeNull();
  });

  it('returns no progress when the condition is absent', () => {
    expect(achProgress(stats, null)).toBeNull();
  });

  it('reports the absolute value once the goal is met, instead of an inverted fraction', () => {
    // 初次觉醒：已记录 12 天、目标 1 天 —— 「12 / 1 天」会读成反的，像是目标写错了
    expect(achProgress(stats, { stat: 'totalDays', operator: '>=', value: 1 })).toEqual({
      text: '已记录 12 天',
      ratio: 1,
    });
    expect(achProgress(stats, { stat: 'maxSkillCheckins', operator: '>=', value: 7 })).toEqual({
      text: '累计打卡 7 天',
      ratio: 1,
    });
  });

  it('keeps the fraction while the goal is still ahead', () => {
    expect(achProgress(stats, { stat: 'totalDays', operator: '>=', value: 100 })).toEqual({
      text: '已记录 12 / 100 天',
      ratio: 0.12,
    });
  });

  it('labels the attribute-count statistic as attribute count, not lit-this-day', () => {
    expect(achProgress(stats, { stat: 'totalAttrs', operator: '>=', value: 8 })).toEqual({
      text: '共 5 / 8 项属性',
      ratio: 0.625,
    });
  });

  it('summarizes all/any composite condition progress', () => {
    expect(
      achProgress(stats, {
        all: [
          { stat: 'totalDays', operator: '>=', value: 12 },
          { stat: 'maxSkillCheckins', operator: '>=', value: 14 },
        ],
      }),
    ).toEqual({ text: '全部条件进度 1 / 2', ratio: 0.75 });
    expect(
      achProgress(stats, {
        any: [
          { stat: 'streak', operator: '>=', value: 7 },
          { stat: 'maxSkillCheckins', operator: '>=', value: 14 },
        ],
      }),
    ).toEqual({ text: '任一条件进度 0 / 2', ratio: 0.5 });
  });
});
