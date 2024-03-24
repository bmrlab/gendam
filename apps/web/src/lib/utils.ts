import classNames from 'classnames'
import { createTwc } from 'react-twc'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: classNames.ArgumentArray) {
  return twMerge(classNames(inputs))
}

export const twx = createTwc({ compose: cn })

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
