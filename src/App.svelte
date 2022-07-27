<script type="ts">
  import { onDestroy, onMount } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api";
  import { listen } from "@tauri-apps/api/event";

  let seconds = 5;
  let timer_tick_unlisten: Promise<UnlistenFn> | null = null;
  let timer_done_unlisten: Promise<UnlistenFn> | null = null;

  type Timer = {
    id: string;
    elapsed: {
      secs: number;
      nsecs: number;
    };
  };

  onMount(() => {
    timer_tick_unlisten = listen<Timer>("timer-tick", (event) => {
      console.log("Tick!", event.payload.id, event.payload.elapsed);
    });

    timer_done_unlisten = listen<Timer>("timer-done", (event) => {
      console.log("Done!", event.payload.id);
    });
  });

  onDestroy(() => {
    timer_tick_unlisten?.then((ttu) => ttu());
    timer_done_unlisten?.then((tdu) => tdu());
  });

  function start_timer() {
    invoke("start_timer", {
      duration: { secs: seconds, nanos: 0 },
      message: "Hi!",
    });
  }
</script>

<main>
  <label>
    Fire after
    <input type="number" bind:value={seconds} />
  </label>
  <button on:click={start_timer}>Fire!</button>
</main>
