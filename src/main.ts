import './app.css'
import App from './App.svelte'

import { invoke } from '@tauri-apps/api'

const app = new App({
  target: document.getElementById('app')
})

invoke('greet', { name: 'world'})
  .then((response) => console.log(response))

export default app
