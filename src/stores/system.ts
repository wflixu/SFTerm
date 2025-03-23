import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { IHost } from '@/shared/interface'
const hostJsonStr = localStorage.getItem('hosts')
const json = hostJsonStr ? JSON.parse(hostJsonStr) : [];
export const useSystemStore = defineStore('system', () => {

  const hosts = ref<IHost[]>(json)
  const currentHost = ref<IHost | null>(null)
  const storeHost = () => {
    localStorage.setItem('hosts', JSON.stringify(hosts.value))
  }
  const setHosts = (newHosts: IHost[]) => {
    hosts.value = newHosts;
    storeHost()
  }
  const setCurrentHost = (host: IHost) => {
    currentHost.value = host
  }
  const addHost = (host: IHost) => {
    hosts.value.push(host)
    storeHost()

  }
  const removeHost = (host: IHost) => {
    const index = hosts.value.findIndex((h) => h.id === host.id)
    if (index !== -1) {
      hosts.value.splice(index, 1)
      storeHost()
    }
  }
  const updateHost = (host: IHost) => {
    const index = hosts.value.findIndex((h) => h.id === host.id)
    if (index !== -1) {
      hosts.value[index] = host
      storeHost()
    }
  }
  const getHost = (id: string) => {
    return hosts.value.find((h) => h.id === id)
  }

  return {
    hosts,
    currentHost,
    setHosts,
    setCurrentHost,
    addHost,
    removeHost,
    updateHost,
    getHost
  }
})
