<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { ref } from 'vue';



const tabs = ref([
  { route: '/', label: 'Home', icon: 'pi pi-home',  },
  { route: '/ftp', label: 'FTP', icon: 'pi pi-chart-line' },
  { route: '/connect', label: 'Connect', icon: 'pi pi-list', closable: true   },
]);

</script>

<template>
  <Tabs value="/" class="tabs">
    <TabList>
      <Tab v-for="tab in tabs" :key="tab.label" :value="tab.label">
        <router-link v-if="tab.route" v-slot="{ href, navigate }" :to="tab.route" custom>
          <a v-ripple :href="href" @click="navigate" class="tab-item">
            <i :class="tab.icon" />
            <span>{{ tab.label }}</span>
            <i class="pi pi-times" v-if="tab.closable"></i>
          </a>
        </router-link>
      </Tab>
    </TabList>
  </Tabs>
  <RouterView />
</template>

<style scoped>

:deep(.p-tablist-tab-list) {
  padding-left: 4rem;
  padding-right: 1rem;
}

:deep(.p-tab) {
  --p-tabs-tab-padding: 0;
}

header {
  line-height: 1.5;
  max-height: 100vh;
}

.tab-item {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  color: var(--color-text);

  span {
    font-size: 16px;
  }
}

.logo {
  display: block;
  margin: 0 auto 2rem;
}

nav {
  width: 100%;
  font-size: 12px;
  text-align: center;
  margin-top: 2rem;
}

nav a.router-link-exact-active {
  color: var(--color-text);
}

nav a.router-link-exact-active:hover {
  background-color: transparent;
}

nav a {
  display: inline-block;
  padding: 0 1rem;
  border-left: 1px solid var(--color-border);
}

nav a:first-of-type {
  border: 0;
}

@media (min-width: 1024px) {
  header {
    display: flex;
    place-items: center;
    padding-right: calc(var(--section-gap) / 2);
  }

  .logo {
    margin: 0 2rem 0 0;
  }

  header .wrapper {
    display: flex;
    place-items: flex-start;
    flex-wrap: wrap;
  }

  nav {
    text-align: left;
    margin-left: -1rem;
    font-size: 1rem;

    padding: 1rem 0;
    margin-top: 1rem;
  }
}
</style>
