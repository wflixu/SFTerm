import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { IHost } from '@/shared/interface'
import { loadState, saveState } from '@/lib/db'

/**
 * 老版本存下来的主机可能没有 id（或者 id 重复）。
 * 没有 id 会让路由 /session/:hostId 变成 /session/，被 404 兜底重定向回 Home，
 * 表现就是「tab 建出来了但点不动」。这里统一补齐并回写。
 */
function normalizeHosts(list: IHost[]): { hosts: IHost[]; changed: boolean } {
  const seen = new Set<string>()
  let changed = false

  const hosts = list
    .filter((host) => host && typeof host === 'object')
    .map((host, index) => {
      let id = host.id
      if (!id || seen.has(id)) {
        id = `host-${Date.now()}-${index}`
        changed = true
      }
      seen.add(id)
      if (id !== host.id) changed = true
      return { ...host, id }
    })

  if (hosts.length !== list.length) changed = true
  return { hosts, changed }
}

export const useSystemStore = defineStore('system', () => {
  const hostJsonStr = window.localStorage.getItem('hosts')
  const parsed: IHost[] = hostJsonStr ? JSON.parse(hostJsonStr) : []
  const { hosts: normalized, changed } = normalizeHosts(parsed)

  const hosts = ref<IHost[]>(normalized)
  const currentHost = ref<IHost | null>(null)

  // 迁移旧凭据时先暂停写库，避免把还带明文密码的中间状态写进 SQLite。
  let persistDb = true
  const setPersistDb = (value: boolean) => {
    persistDb = value
  }

  const storeHost = () => {
    window.localStorage.setItem('hosts', JSON.stringify(hosts.value))
    if (persistDb) {
      saveState('hosts', hosts.value).catch((error) =>
        console.error('[system] 写入 SQLite 失败', error),
      )
    }
  }

  /** 首次启动时把 localStorage 里的主机导入 SQLite；之后以数据库为准。 */
  const hydrate = async () => {
    const stored = await loadState<IHost[]>('hosts')
    if (stored) {
      const normalized = normalizeHosts(stored)
      hosts.value = normalized.hosts
      if (normalized.changed) storeHost()
      return
    }
    if (hosts.value.length) await saveState('hosts', hosts.value)
  }

  const persist = () => storeHost()

  if (changed) storeHost()

  const setHosts = (newHosts: IHost[]) => {
    hosts.value = normalizeHosts(newHosts).hosts
    storeHost()
  }

  const setCurrentHost = (host: IHost) => {
    currentHost.value = host
  }

  const addHost = (host: IHost) => {
    const withId = host.id ? host : { ...host, id: `host-${Date.now()}` }
    hosts.value.push(withId)
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
    getHost,
    hydrate,
    persist,
    setPersistDb
  }
})
