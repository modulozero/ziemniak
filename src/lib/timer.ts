// Copyright 2022 ModZero.
// SPDX-License-Identifier: 	AGPL-3.0-or-later

import { invoke } from "@tauri-apps/api";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import {
  writable,
  type Readable,
  type Subscriber,
  type Unsubscriber,
} from "svelte/store";

export type Duration = {
  readonly secs: number;
  readonly nanos: number;
};

export type TimerData = {
  readonly id: string;
  readonly duration: Duration;
  readonly elapsed: Duration | null;
  readonly version: number;
};

export class Timer implements Readable<TimerData> {
  private subscribers: Set<Subscriber<TimerData>> = new Set();

  private _data: TimerData | null = null;
  private get data(): TimerData | null {
    return this._data;
  }
  private set data(v: TimerData | null) {
    this._data = v;
    this.subscribers.forEach((s) => s(v));
  }

  x = writable(0);

  public subscribe(run: Subscriber<TimerData>): Unsubscriber {
    run(this.data);
    this.subscribers.add(run);

    return () => {
      this.subscribers.delete(run);
    };
  }

  private ensureReady() {
    if (!this.ready) {
      throw new Error("Timer Still Processing");
    }
  }

  private _duration: Duration | null;
  public get duration(): Duration | null {
    return this._duration;
  }

  public reset(duration: Duration) {
    this.ensureReady();
    invoke("plugin:timers|reset", {
      timerId: this.data.id,
      duration: duration,
    });
  }

  private _elapsed: Duration | null;
  public get elapsed(): Duration | null {
    return this._elapsed;
  }

  private unlistenUpdate: Promise<UnlistenFn> | null = null;

  private onUpdate(event: Event<TimerData>): void {
    if (
      event.payload.id === this.data.id &&
      event.payload.version >= this.data.version
    ) {
      this.data = event.payload;
    }
  }

  public get ready() {
    return this.data !== null;
  }

  constructor(duration: Duration) {
    this.unlistenUpdate = listen<TimerData>("timer-update", (event) =>
      this.onUpdate(event)
    );

    invoke<TimerData>("plugin:timers|make", {
      duration: duration,
    }).then((timer) => {
      this.data = timer;
    });
  }

  public close() {
    if (this.ready) {
      invoke("plugin:timers|delete", { timerId: this.data.id });
    }

    this.unlistenUpdate.then((u) => u());
  }

  public start() {
    this.ensureReady();
    invoke("plugin:timers|start", { timerId: this.data.id });
  }
}
