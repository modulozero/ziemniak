<!--
 Copyright 2022 ModZero.
 SPDX-License-Identifier: 	AGPL-3.0-or-later
-->
<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { Timer } from "@app/lib/timer";

  let seconds = 5;
  let timer: Timer = new Timer({ secs: seconds, nanos: 0 });
  $: timer.ready && timer.reset({ secs: seconds, nanos: 0 });
  $: console.log(seconds);

  onDestroy(() => {
    timer.close();
  });
</script>

<div class="timer" />
<div class="meter">
  {#if $timer !== null}
    {$timer.elapsed ? `${$timer.elapsed.secs}.${$timer.elapsed.nanos}` : `0.0`} /
    {$timer.duration.secs}
  {:else}
    ...
  {/if}
</div>

<div class="controls">
  <label>
    Fire after
    <input
      disabled={$timer === null || !!$timer.elapsed}
      type="number"
      bind:value={seconds}
    />
  </label>
  <button
    disabled={$timer === null || !!$timer.elapsed}
    on:click={() => timer.start()}>Fire!</button
  >
  <button
    disabled={$timer === null}
    on:click={() => timer.reset({ secs: seconds, nanos: 0 })}>Reset</button
  >
</div>

<style>
  .timer {
    display: flex;
    flex-direction: column;
  }
  .meter {
    flex-grow: 1;
    margin: 0.5em;
  }

  .controls {
    margin: 0.5em;
    padding: 0.5em;
  }
</style>
