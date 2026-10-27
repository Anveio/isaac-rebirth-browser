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
      const price = pickup.shopPrice?.sprite;
      if (price) {
        //SetAnimation owns global frame-1/stopped; actual independent
        //SetLayerFrame overrides select each static currency/digit layer.
        for (const [layer, frame] of price.layerFrames.entries()) {
          if (frame === null) continue;
          this.assets.draw(ctx, price.actor, price.animation, x, y, frame, { layers: [layer], overrides });
        }
      }
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
    const using = state.playerAnimation.current === "UseItem";
    if ((!owner?.pickupAnimationGuard && !using) || !presentation) return;
    const attachment = this.assets.nullFrame("gfx/001.000_player.anm2", using ? "UseItem" : "Pickup", 0,
      using ? state.playerAnimation.frame : owner!.pickupAnimationFrame);
    if (presentation.regular) {
      const copy = presentation.regular;
      const [RedTint, GreenTint, BlueTint] = state.stageEffects.actorTint;
      //50A830 copied the source local pose after this turn's Player
      //callback.533970 changes it only at the next actual Player callback;
      //that birth frame must retain the visible Idle layer at its old pose.
      if (copy.attachedToPlayer && !attachment?.Visible) return;
      //533970 attaches every copied layer: the null position goes to each
      //LayerState+0x30 offset (added to the keyframe) and its visibility to
      //the layer's +0x64 flag. Supported regular actors use body0 plus
      //optional glow1; preserve both actual authored channels.
      const layerOffset = copy.attachedToPlayer && attachment
        ? { x: attachment.XPosition, y: attachment.YPosition } : undefined;
      const overrides = Object.fromEntries([0, 1].map(id => [id, { RedTint, GreenTint, BlueTint }]));
      this.assets.draw(ctx, copy.actor, copy.animation, x, y, copy.animationFrame, { overrides, layerOffset });
      if (copy.overlay) this.assets.draw(ctx, copy.actor, copy.overlay, x, y, copy.overlayFrame, { overrides, layerOffset });
      return;
    }
    if (!attachment?.Visible) return;
    if (presentation.pocket) {
      const [RedTint, GreenTint, BlueTint] = state.stageEffects.actorTint;
      const overrides = Object.fromEntries(Array.from({length: 3}, (_, layer) => [layer, { RedTint, GreenTint, BlueTint }]));
      const layerOffset = { x: attachment.XPosition, y: attachment.YPosition };
      //The one copied7E0 sprite has a Sparkle main and a static named
      //overlay. The whole main animation renders first and the overlay on
      //top of it, so the StarFlash (Sparkle layer1) shows only around the
      //edges of the pill, rune or card face, as in the original footage.
      this.assets.draw(ctx, "gfx/ui/ui_cardspills.anm2", "Sparkle", x, y, presentation.animationFrame, {
        overrides, layerOffset,
      });
      this.assets.draw(ctx, "gfx/ui/ui_cardspills.anm2", presentation.pocket.overlay, x, y, presentation.pocket.overlayFrame, {
        overrides, layerOffset,
      });
      return;
    }
    const item = catalog.items.find(item => item.id === presentation.itemId && item.kind !== "trinket");
    if (!item?.sprite) return;
    const [RedTint, GreenTint, BlueTint] = state.stageEffects.actorTint;
    // 533970 adds null0's position to every copied layer's LayerState+0x30
    // offset and copies its visibility to the layer (+0x64), leaving each
    // keyframe's own position and visibility (the sparkle's -16,-16 and
    // hidden first frame) in force; 5099ca then renders that copied sprite
    // at the Player render anchor.
    this.assets.draw(ctx, "gfx/005.100_collectible.anm2", "PlayerPickupSparkle", x, y, presentation.animationFrame, {
      spritesheets: {1: item.sprite},
      overrides: Object.fromEntries(Array.from({length: 4}, (_, layer) => [layer, { RedTint, GreenTint, BlueTint }])),
      layerOffset: { x: attachment.XPosition, y: attachment.YPosition },
    });
  }
}
