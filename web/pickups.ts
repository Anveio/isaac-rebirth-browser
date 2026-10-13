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
      if (pickup.overlay) {
        // Native collectible Init selects Alternates overlay frame 0 for
        // the pedestal, while its main Idle clip animates the item above it.
        this.assets.draw(ctx, pickup.actor, pickup.overlay, x, y, pickup.overlayFrame, { layers: [0], overrides });
      }
      this.assets.draw(ctx, pickup.actor, pickup.animation, x, y, pickup.animationFrame, {
        spritesheets, overrides,
      });
    }
  }

  private itemSprite(pickup: Pickup): Record<number, string> | undefined {
    if (pickup.variant !== 100 && pickup.variant !== 350) return;
    if (pickup.variant === 100 && pickup.spritesheetItemId === 0) return;
    const trinket = pickup.variant === 350;
    const id = pickup.variant === 100 ? pickup.spritesheetItemId : pickup.subtype;
    const item = catalog.items.find(item => item.id === id && (item.kind === "trinket") === trinket);
    if (!item?.sprite) throw new Error(`Missing authored item art ${pickup.variant}/${pickup.subtype}`);
    return { [trinket ? 0 : 1]: item.sprite };
  }

  drawCarried(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    const owner = state.playerItems;
    const presentation = owner?.pickupPresentation;
    if (!owner?.pickupAnimationGuard || !presentation) return;
    const attachment = this.assets.nullFrame("gfx/001.000_player.anm2", "Pickup", 0, owner.pickupAnimationFrame);
    if (!attachment?.Visible) return;
    const item = catalog.items.find(item => item.id === presentation.itemId && item.kind !== "trinket");
    if (!item?.sprite) return;
    const [RedTint, GreenTint, BlueTint] = state.stageEffects.actorTint;
    // 533970 copies null0 position/visibility into every copied sprite layer;
    // 5099ca then renders that copied sprite at the Player render anchor.
    this.assets.draw(ctx, "gfx/005.100_collectible.anm2", "PlayerPickupSparkle", x, y, presentation.animationFrame, {
      spritesheets: {1: item.sprite},
      overrides: Object.fromEntries(Array.from({length: 4}, (_, layer) => [layer, {
        XPosition: attachment.XPosition, YPosition: attachment.YPosition,
        Visible: attachment.Visible, RedTint, GreenTint, BlueTint,
      }])),
    });
  }
}
