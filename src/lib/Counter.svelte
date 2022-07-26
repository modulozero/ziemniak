<script lang="ts">
  let count: number = 0
  $: doubled = count * 2
  const increment = () => {
    count += 1
  }
</script>

<button on:click={increment}>
  count is {count} and doubled is {doubled}
</button>
