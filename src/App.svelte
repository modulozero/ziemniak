<script>
  import { onDestroy } from 'svelte'
  import { invoke } from '@tauri-apps/api'
  import { listen, once } from '@tauri-apps/api/event'
  
  let seconds = 5

  function start_timer() {
    invoke('start_timer', {
      duration: { secs: seconds, nanos: 0 },
      message: "Hi!",
    })

    let timer_tick_unlisten = listen('timer-tick', (event) => {
      console.log("Tick!", event.payload.id, event.payload.elapsed)
    })

    once('timer-done', (event) => {
      console.log("Done!", event.payload.id)

      timer_tick_unlisten.then(ttu => ttu())
    })
  }
</script>

<main>
  <label>
    Fire after
    <input type="number" bind:value={seconds} />
  </label>
  <button on:click="{start_timer}">Fire!</button>
</main>
