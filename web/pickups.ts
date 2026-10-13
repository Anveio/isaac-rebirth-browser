import { Assets } from "./animation";
import catalog from "./pickup-data.json";
import type { Pickup, State } from "./engine";

export class PickupRenderer {
  constructor(private readonly assets: Assets) {}

  draw(ctx: CanvasRenderingContext2D, state: State): void {
    const [RedTint, GreenTint, BlueTint] = state.stageEffects.actorTint;
    const overrides = Object.fromEntries(Array.from({ length: 4 }, (_, id) => [id, { RedTint, GreenTint, BlueTint }]));
    for (const pickup of state.room.pickups) {
      if (pickup.removed) continue;
      const x = Math.round(pickup.x * 2) / 2, y = Math.round((pickup.y - 1) * 2) / 2;
      const spritesheets = this.itemSprite(pickup);
      if (pickup.variant === 100) {
        // Native collectible Init selects Alternates overlay frame 0 for
        // the pedestal, while its main Idle clip animates the item above it.
        this.assets.draw(ctx, pickup.actor, "Alternates", x, y, 0, { layers: [0], overrides });
      }
      this.assets.draw(ctx, pickup.actor, pickup.animation, x, y, pickup.animationFrame, {
        spritesheets, overrides,
      });
    }
  }

  private itemSprite(pickup: Pickup): Record<number, string> | undefined {
    if (pickup.variant !== 100 && pickup.variant !== 350) return;
    if (pickup.variant === 100 && pickup.subtype === 0) return;
    const trinket = pickup.variant === 350;
    const item = catalog.items.find(item => item.id === pickup.subtype && (item.kind === "trinket") === trinket);
    if (!item?.sprite) throw new Error(`Missing authored item art ${pickup.variant}/${pickup.subtype}`);
    return { [trinket ? 0 : 1]: item.sprite };
  }
}
