import { createApp } from 'vue'
import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config';
import Aura from '@primeuix/themes/aura';
import 'primeicons/primeicons.css'
import '@xterm/xterm/css/xterm.css'

import App from './App.vue'
import router from './router'
import './assets/main.css'
import setupUILib from './shared/uilib';

const app = createApp(App)
app.use(PrimeVue, {
    theme: {
        preset: Aura
    }
});
app.use(setupUILib)
app.use(createPinia())
app.use(router)

app.mount('#app')
