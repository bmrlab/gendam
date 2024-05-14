/**
 * 一些复杂的 icons
 */

import { cn } from '@gendam/tailwind/utils'
import { HTMLAttributes } from 'react'
import './style.css'

const FlashStroke = ({
  bold,
  className,
  ...props
}: {
  bold?: boolean
} & HTMLAttributes<HTMLDivElement>) => (
  <div {...props} className={cn('relative h-4 w-4', className)}>
    <div
      className={cn(
        'absolute left-0 top-0 h-full w-full rounded-full',
        'repeat-infinite animate-[flashstroke] border-current',
        bold ? 'border-2' : 'border',
      )}
      style={{ animationDuration: '2s', animationDelay: '0s' }}
    ></div>
    <div
      className={cn(
        'absolute left-0 top-0 h-full w-full rounded-full',
        'repeat-infinite animate-[flashstroke] border-current',
        bold ? 'border-2' : 'border',
      )}
      style={{ animationDuration: '2s', animationDelay: '1s' }}
    ></div>
  </div>
)

export { FlashStroke }
