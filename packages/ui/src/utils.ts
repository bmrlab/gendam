import classNames from 'classnames'
import { createTwc } from 'react-twc'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: classNames.ArgumentArray) {
  return twMerge(classNames(inputs))
}

export const twx = createTwc({ compose: cn })
