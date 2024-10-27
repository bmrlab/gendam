export { cn, twx } from '@gendam/tailwind/utils'
export { confirm } from './ConfirmDialog'

export function formatDuration(seconds: number) {
  let d = seconds < 0 ? 0 : seconds
  var h = Math.floor(d / 3600)
  var m = Math.floor((d % 3600) / 60)
  var s = Math.floor((d % 3600) % 60)
  var hDisplay = h > 0 ? (h < 10 ? '0' + h : h) + ':' : '00:'
  var mDisplay = m > 0 ? (m < 10 ? '0' + m : m) + ':' : '00:'
  var sDisplay = s > 0 ? (s < 10 ? '0' + s : s) : '00'
  return hDisplay + mDisplay + sDisplay
}

export function formatBytes(bytes: number, decimals: number = 2) {
  bytes = bytes < 0 ? 0 : bytes
  decimals = decimals < 0 ? 0 : decimals
  if (bytes === 0) {
    return '0 Bytes'
  }
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(decimals)) + ' ' + sizes[i]
}

export function formatDateTime(date: string) {
  // format datetime as 'YYYY-MM-DD hh::mm::ss', padding with 0
  const pad = (n: number): string => (n < 10 ? '0' : '') + n
  const d = new Date(date)
  const year = d.getFullYear()
  const month = d.getMonth() + 1
  const day = d.getDate()
  const hour = d.getHours()
  const minute = d.getMinutes()
  const second = d.getSeconds()
  return `${year}-${pad(month)}-${pad(day)} ${pad(hour)}:${pad(minute)}:${pad(second)}`
}

export function timeToSeconds(time: string) {
  const parts = time.split(':').map(Number) // 将字符串按冒号分割并转换为数字
  let seconds = 0

  // 处理不同长度的时间格式
  if (parts.length === 3) {
    // "hh:mm:ss" 格式
    let hours = parts[0]
    let minutes = parts[1]
    seconds = parts[2]
    return hours * 3600 + minutes * 60 + seconds
  } else if (parts.length === 2) {
    // "mm:ss" 格式
    let minutes = parts[0]
    seconds = parts[1]
    return minutes * 60 + seconds
  } else if (parts.length === 1) {
    // "ss" 格式
    return parts[0]
  } else {
    // 无效格式
    throw new Error('Invalid time format')
  }
}
