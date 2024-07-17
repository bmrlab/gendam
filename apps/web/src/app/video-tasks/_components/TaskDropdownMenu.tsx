import Icon from '@gendam/ui/icons'
import classNames from 'classnames'
import { DropdownMenu } from '@gendam/ui/v2/dropdown-menu'
// import { cn } from '@/lib/utils'
import { PropsWithChildren, ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

export type DropdownMenuOptions =
  | {
      disabled?: boolean
      variant?: 'accent' | 'destructive'
      label: string | ReactNode
      handleSelect: () => void
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
  const { t } = useTranslation()
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
            <span className="sr-only">{t('task.dropdown.openMenu')}</span>
            {triggerIcon ? triggerIcon : <Icon.MoreVertical />}
          </div>
        )}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end">
        {options.map((o, index) => (
          o === 'Separator' ? (
            <DropdownMenu.Separator key={index} className="bg-app-line h-px my-1" />
          ) : (
            <DropdownMenu.Item key={index} onSelect={o.handleSelect} variant={o.variant} disabled={o.disabled}>
              {o.label}
            </DropdownMenu.Item>
          )
        ))}
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  )
}
