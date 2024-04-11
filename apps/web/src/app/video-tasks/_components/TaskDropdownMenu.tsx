import Icon from '@/components/Icon'
import classNames from 'classnames'
import { DropdownMenu } from '@muse/ui/v2/dropdown-menu'
import { cn } from '@/lib/utils'
import { PropsWithChildren, ReactNode } from 'react'

export type DropdownMenuOptions =
  | {
      disabled?: boolean
      variant?: 'accent' | 'destructive'
      label: string | ReactNode
      handleClick: () => void
    }
  | 'Separator'

type _DropdownMenuProps = {
  triggerIcon?: ReactNode
  options: Array<DropdownMenuOptions>
  contentClassName?: string
}

export default function TaskDropdownMenu({
  options,
  triggerIcon,
  contentClassName,
  children,
}: PropsWithChildren<_DropdownMenuProps>) {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        {children ? (
          children
        ) : (
          <div className={classNames(
            'inline-flex items-center justify-center size-6 rounded border border-app-line',
            'cursor-default data-[state=open]:bg-app-hover',
          )}>
            <span className="sr-only">Open menu</span>
            {triggerIcon ? triggerIcon : <Icon.more />}
          </div>
        )}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end" className={cn('', contentClassName)}>
        {options.map((o, index) => (
          <div key={index}>
            {o === 'Separator' ? (
              <DropdownMenu.Separator className="bg-app-line h-px my-1" />
            ) : (
              <DropdownMenu.Item key={index} onClick={o.handleClick} variant={o.variant} disabled={o.disabled}>
                {o.label}
              </DropdownMenu.Item>
            )}
          </div>
        ))}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  )
}
