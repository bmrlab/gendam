import Icon from '@/components/Icon'
import classNames from 'classnames'
import { DropdownMenu } from '@gendam/ui/v2/dropdown-menu'
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
}

export default function TaskDropdownMenu({
  options,
  triggerIcon,
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
      <DropdownMenu.Content align="end">
        {options.map((o, index) => (
          o === 'Separator' ? (
            <DropdownMenu.Separator key={index} className="bg-app-line h-px my-1" />
          ) : (
            <DropdownMenu.Item key={index} onClick={o.handleClick} variant={o.variant} disabled={o.disabled}>
              {o.label}
            </DropdownMenu.Item>
          )
        ))}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  )
}
