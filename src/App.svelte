<!--
 Copyright 2022 ModZero.
 SPDX-License-Identifier: 	AGPL-3.0-or-later
-->
<script type="ts">
  import { onDestroy, onMount } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api";
  import { listen } from "@tauri-apps/api/event";

  let seconds = 5;
  let timerTickUnlisten: Promise<UnlistenFn> | null = null;
  let timerDoneUnlisten: Promise<UnlistenFn> | null = null;

  type Timer = {
    id: string;
    elapsed: {
      secs: number;
      nsecs: number;
    };
  };

  onMount(() => {
    timerTickUnlisten = listen<Timer>("timer-tick", (event) => {
      console.log("Tick!", event.payload.id, event.payload.elapsed);
    });

    timerDoneUnlisten = listen<Timer>("timer-done", (event) => {
      console.log("Done!", event.payload.id);
    });
    
  });

  onDestroy(() => {
    timerTickUnlisten?.then((ttu) => ttu());
    timerDoneUnlisten?.then((tdu) => tdu());
  });

  async function startTimer() {
    let timer = await invoke<Timer>("make_timer", {
      duration: { secs: seconds, nanos: 0 },
      message: "Hi!",
    });
    invoke("start_timer", { timerId: timer.id });
  }
</script>

<main>
  <div id="timer" />
  <div id="controls">
    <label>
      Fire after
      <input type="number" bind:value={seconds} />
    </label>
    <button on:click={startTimer}>Fire!</button>
  </div>
</main>

<style>
  main {
    position: fixed;
    top: 0px;
    left: 0px;
    right: 0px;
    bottom: 0px;
    display: flex;
    flex-direction: column;
  }
  #timer {
    flex-grow: 1;
    margin: 0.5em;
  }

  #controls {
    margin: 0.5em;
    padding: 0.5em;
  }
</style>
