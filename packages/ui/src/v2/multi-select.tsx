'use client'
import { cn } from '@gendam/tailwind/utils'
import { CaretSortIcon, CheckIcon, Cross2Icon } from '@radix-ui/react-icons'
import * as PopoverPrimitive from '@radix-ui/react-popover'
import { Primitive, type ComponentPropsWithoutRef } from '@radix-ui/react-primitive'
import { useControllableState } from '@radix-ui/react-use-controllable-state'
import {
  ElementRef,
  FC,
  ReactNode,
  createContext,
  forwardRef,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from 'react'
import { createPortal } from 'react-dom'
import { Tooltip } from '../v2/tooltip'
import { Badge } from './badge'
import { CommandPrimitive } from './command'

export interface MultiSelectOptionItem {
  value: string
  label?: React.ReactNode
}

interface MultiSelectContextValue {
  value: string[]
  open: boolean
  onSelect(value: string, item: MultiSelectOptionItem): void
  onDeselect(value: string, item: MultiSelectOptionItem): void
  onSearch?(keyword: string | undefined): void
  filter?: boolean | ((keyword: string, current: string) => boolean)
  disabled?: boolean
  maxCount?: number
  itemCache: Map<string, MultiSelectOptionItem>
}

const MultiSelectContext = createContext<MultiSelectContextValue | undefined>(undefined)

const useMultiSelect = () => {
  const context = useContext(MultiSelectContext)

  if (!context) {
    throw new Error('useMultiSelect must be used within MultiSelectProvider')
  }

  return context
}

export type MultiSelectProps = ComponentPropsWithoutRef<typeof PopoverPrimitive.Root> & {
  value?: string[]
  onValueChange?(value: string[], items: MultiSelectOptionItem[]): void
  onSelect?(value: string, item: MultiSelectOptionItem): void
  onDeselect?(value: string, item: MultiSelectOptionItem): void
  defaultValue?: string[]
  onSearch?(keyword: string | undefined): void
  filter?: boolean | ((keyword: string, current: string) => boolean)
  disabled?: boolean
  maxCount?: number
}

const MultiSelectRoot: FC<MultiSelectProps> = ({
  value: valueProp,
  onValueChange: onValueChangeProp,
  onDeselect: onDeselectProp,
  onSelect: onSelectProp,
  defaultValue,
  open: openProp,
  onOpenChange,
  defaultOpen,
  onSearch,
  filter,
  disabled,
  maxCount,
  ...popoverProps
}) => {
  const itemCache = useRef(new Map<string, MultiSelectOptionItem>()).current

  const handleValueChange = useCallback(
    (state: string[]) => {
      if (onValueChangeProp) {
        const items = state.map((value) => itemCache.get(value)!)

        onValueChangeProp(state, items)
      }
    },
    [onValueChangeProp, itemCache],
  )

  const [value, setValue] = useControllableState({
    prop: valueProp,
    defaultProp: defaultValue,
    onChange: handleValueChange,
  })

  const [open, setOpen] = useControllableState({
    prop: openProp,
    defaultProp: defaultOpen,
    onChange: onOpenChange,
  })

  const handleSelect = useCallback(
    (value: string, item: MultiSelectOptionItem) => {
      setValue((prev) => {
        if (prev?.includes(value)) {
          return prev
        }

        onSelectProp?.(value, item)

        return prev ? [...prev, value] : [value]
      })
    },
    [onSelectProp, setValue],
  )

  const handleDeselect = useCallback(
    (value: string, item: MultiSelectOptionItem) => {
      setValue((prev) => {
        if (!prev || !prev.includes(value)) {
          return prev
        }

        onDeselectProp?.(value, item)

        return prev.filter((v) => v !== value)
      })
    },
    [onDeselectProp, setValue],
  )

  const contextValue = useMemo(() => {
    return {
      value: value || [],
      open: open || false,
      onSearch,
      filter,
      disabled,
      maxCount,
      onSelect: handleSelect,
      onDeselect: handleDeselect,
      itemCache,
    }
  }, [value, open, onSearch, filter, disabled, maxCount, handleSelect, handleDeselect, itemCache])

  return (
    <MultiSelectContext.Provider value={contextValue}>
      <PopoverPrimitive.Root {...popoverProps} open={open} onOpenChange={setOpen} />
    </MultiSelectContext.Provider>
  )
}

MultiSelectRoot.displayName = 'MultiSelectRoot'

type MultiSelectTriggerElement = React.ElementRef<typeof Primitive.div>

interface MultiSelectTriggerProps extends ComponentPropsWithoutRef<typeof Primitive.div> {
  openClassName?: string
  icon?: (open: boolean) => ReactNode
}

const PreventClick = (e: React.MouseEvent | React.TouchEvent) => {
  e.preventDefault()
  e.stopPropagation()
}

const MultiSelectTrigger = forwardRef<MultiSelectTriggerElement, MultiSelectTriggerProps>(
  ({ icon, openClassName, className, children, ...props }, forwardedRef) => {
    const { disabled, open } = useMultiSelect()

    return (
      <PopoverPrimitive.Trigger ref={forwardedRef as any} asChild>
        <div
          aria-disabled={disabled}
          data-disabled={disabled}
          {...props}
          className={cn(
            'border-input ring-offset-background focus:ring-ring flex h-full min-h-9 w-full items-center justify-between whitespace-nowrap rounded-md border bg-transparent px-3 py-2 text-sm shadow-sm focus:outline-none focus:ring-1 [&>span]:line-clamp-1',
            disabled ? 'cursor-not-allowed opacity-50' : 'cursor-text',
            className,
            open && openClassName,
          )}
          onClick={disabled ? PreventClick : props.onClick}
          onTouchStart={disabled ? PreventClick : props.onTouchStart}
        >
          {children}
          {icon ? icon(open) : <CaretSortIcon aria-hidden className="h-4 w-4 shrink-0 opacity-50" />}
        </div>
      </PopoverPrimitive.Trigger>
    )
  },
)

MultiSelectTrigger.displayName = 'MultiSelectTrigger'

interface MultiSelectValueProps extends ComponentPropsWithoutRef<typeof Primitive.div> {
  placeholder?: string
  placeholderClassName?: string
  maxDisplay?: number
  maxItemLength?: number
  badge?: (key: string, children: unknown) => ReactNode
}

// eslint-disable-next-line react/display-name
const MultiSelectValue = forwardRef<ElementRef<typeof Primitive.div>, MultiSelectValueProps>(
  ({ badge, className, placeholder, placeholderClassName, maxDisplay, maxItemLength, ...props }, forwardRef) => {
    const { value, itemCache, onDeselect } = useMultiSelect()
    const [firstRendered, setFirstRendered] = useState(false)

    const renderRemain = maxDisplay && value.length > maxDisplay ? value.length - maxDisplay : 0
    const renderItems = renderRemain ? value.slice(0, maxDisplay) : value

    useLayoutEffect(() => {
      setFirstRendered(true)
    }, [])

    if (!value.length || !firstRendered) {
      return (
        <span className={cn('text-muted-foreground pointer-events-none', placeholderClassName)}>{placeholder}</span>
      )
    }

    return (
      <Tooltip.Provider delayDuration={300}>
        <div
          className={cn('flex flex-1 flex-wrap items-center gap-1.5 overflow-x-hidden', className)}
          {...props}
          ref={forwardRef}
        >
          {renderItems.map((value) => {
            const item = itemCache.get(value)

            const content = item?.label || value

            const child =
              maxItemLength && typeof content === 'string' && content.length > maxItemLength
                ? `${content.slice(0, maxItemLength)}...`
                : content

            const el = badge ? (
              badge(value, child)
            ) : (
              <Badge
                variant="outline"
                key={value}
                className="group/multi-select-badge cursor-pointer rounded-full px-2 py-0 pr-1 leading-4"
                onClick={(e) => {
                  e.preventDefault()
                  e.stopPropagation()
                  onDeselect(value, item!)
                }}
              >
                <span>{child}</span>
                <Cross2Icon className="text-muted-foreground group-hover/multi-select-badge:text-foreground ml-1 h-3 w-3" />
              </Badge>
            )

            if (child !== content) {
              return (
                <Tooltip.Root key={value}>
                  <Tooltip.Trigger className="inline-flex">{el}</Tooltip.Trigger>
                  <Tooltip.Content side="bottom" align="start" className="z-[51]">
                    {content}
                  </Tooltip.Content>
                </Tooltip.Root>
              )
            }

            return el
          })}
          {renderRemain ? <span className="py-.5 text-muted-foreground text-xs leading-4">+{renderRemain}</span> : null}
        </div>
      </Tooltip.Provider>
    )
  },
)

const MultiSelectSearch = forwardRef<
  ElementRef<typeof CommandPrimitive.Input>,
  ComponentPropsWithoutRef<typeof CommandPrimitive.Input>
>((props, ref) => {
  const { onSearch } = useMultiSelect()

  return <CommandPrimitive.Input ref={ref} {...props} onValueChange={onSearch} />
})

MultiSelectSearch.displayName = 'MultiSelectSearch'

const MultiSelectList = forwardRef<
  React.ElementRef<typeof CommandPrimitive.List>,
  ComponentPropsWithoutRef<typeof CommandPrimitive.List>
>(({ className, ...props }, ref) => {
  return <CommandPrimitive.List ref={ref} className={cn('max-h-[unset] px-0 py-1', className)} {...props} />
})

MultiSelectList.displayName = 'MultiSelectList'

interface MultiSelectContentProps extends ComponentPropsWithoutRef<typeof PopoverPrimitive.Content> {
  contentClassName?: string
}

// eslint-disable-next-line react/display-name
const MultiSelectContent = forwardRef<ElementRef<typeof PopoverPrimitive.Content>, MultiSelectContentProps>(
  ({ contentClassName, className, children, ...props }, ref) => {
    const context = useMultiSelect()

    const fragmentRef = useRef<DocumentFragment>()

    if (!fragmentRef.current && typeof window !== 'undefined') {
      fragmentRef.current = document.createDocumentFragment()
    }

    if (!context.open) {
      return fragmentRef.current
        ? createPortal(<CommandPrimitive>{children}</CommandPrimitive>, fragmentRef.current)
        : null
    }

    return (
      <PopoverPrimitive.Portal forceMount>
        <PopoverPrimitive.Content
          ref={ref}
          align="start"
          sideOffset={4}
          collisionPadding={10}
          className={cn(
            'bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-full rounded-md border p-0 shadow-md outline-none',
            contentClassName,
          )}
          style={
            {
              '--radix-select-content-transform-origin': 'var(--radix-popper-transform-origin)',
              '--radix-select-content-available-width': 'var(--radix-popper-available-width)',
              '--radix-select-content-available-height': 'var(--radix-popper-available-height)',
              '--radix-select-trigger-width': 'var(--radix-popper-anchor-width)',
              '--radix-select-trigger-height': 'var(--radix-popper-anchor-height)',
            } as any
          }
          {...props}
        >
          <CommandPrimitive
            className={cn('max-h-96 w-full min-w-[var(--radix-select-trigger-width)] px-1', className)}
            shouldFilter={!context.onSearch}
          >
            {children}
          </CommandPrimitive>
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    )
  },
)

type MultiSelectItemProps = ComponentPropsWithoutRef<typeof CommandPrimitive.Item> &
  Partial<MultiSelectOptionItem> & {
    checkIcon?: ReactNode
    onSelect?: (value: string, item: MultiSelectOptionItem) => void
    onDeselect?: (value: string, item: MultiSelectOptionItem) => void
  }

// eslint-disable-next-line react/display-name
const MultiSelectItem = forwardRef<ElementRef<typeof CommandPrimitive.Item>, MultiSelectItemProps>(
  (
    {
      value,
      onSelect: onSelectProp,
      onDeselect: onDeselectProp,
      children,
      label,
      disabled: disabledProp,
      checkIcon,
      className,
      ...props
    },
    forwardedRef,
  ) => {
    const { value: contextValue, maxCount, onSelect, onDeselect, itemCache } = useMultiSelect()

    const item = useMemo(() => {
      return value
        ? {
            value,
            label: label || (typeof children === 'string' ? children : undefined),
          }
        : undefined
    }, [value, label, children])

    const selected = Boolean(value && contextValue.includes(value))

    useEffect(() => {
      if (value) {
        itemCache.set(value, item!)
      }
    }, [selected, value, item, itemCache])

    const disabled = Boolean(disabledProp || (!selected && maxCount && contextValue.length >= maxCount))

    const handleClick = () => {
      if (selected) {
        onDeselectProp?.(value!, item!)
        onDeselect(value!, item!)
      } else {
        itemCache.set(value!, item!)
        onSelectProp?.(value!, item!)
        onSelect(value!, item!)
      }
    }

    return (
      <CommandPrimitive.Item
        {...props}
        value={value}
        className={cn(disabled && 'text-muted-foreground cursor-not-allowed', 'flex items-center', className)}
        disabled={disabled}
        onSelect={!disabled && value ? handleClick : undefined}
        ref={forwardedRef}
      >
        <span className="mr-2 overflow-hidden text-ellipsis whitespace-nowrap">{children || label || value}</span>
        {selected ? checkIcon ? checkIcon : <CheckIcon className="ml-auto h-4 w-4 shrink-0" /> : null}
      </CommandPrimitive.Item>
    )
  },
)

const MultiSelectGroup = forwardRef<
  React.ElementRef<typeof CommandPrimitive.Group>,
  ComponentPropsWithoutRef<typeof CommandPrimitive.Group>
>((props, forwardRef) => {
  return <CommandPrimitive.Group {...props} ref={forwardRef} />
})

MultiSelectGroup.displayName = 'MultiSelectGroup'

const MultiSelectSeparator = forwardRef<
  React.ElementRef<typeof CommandPrimitive.Separator>,
  ComponentPropsWithoutRef<typeof CommandPrimitive.Separator>
>((props, forwardRef) => {
  return <CommandPrimitive.Separator {...props} ref={forwardRef} />
})

MultiSelectSeparator.displayName = 'MultiSelectSeparator'

const MultiSelectEmpty = forwardRef<
  ElementRef<typeof CommandPrimitive.Empty>,
  ComponentPropsWithoutRef<typeof CommandPrimitive.Empty>
>(({ children = 'No Content', ...props }, forwardRef) => {
  return (
    <CommandPrimitive.Empty {...props} ref={forwardRef}>
      {children}
    </CommandPrimitive.Empty>
  )
})

MultiSelectEmpty.displayName = 'MultiSelectEmpty'

export interface MultiSelectOptionSeparator {
  type: 'separator'
}

export interface MultiSelectOptionGroup {
  heading?: React.ReactNode
  value?: string
  children: MultiSelectOption[]
}

export type MultiSelectOption =
  | Pick<MultiSelectItemProps, 'value' | 'label' | 'disabled' | 'onSelect' | 'onDeselect'>
  | MultiSelectOptionSeparator
  | MultiSelectOptionGroup

const renderMultiSelectOptions = (list: MultiSelectOption[]) => {
  return list.map((option, index) => {
    if ('type' in option) {
      if (option.type === 'separator') {
        return <MultiSelectSeparator key={index} />
      }

      return null
    }

    if ('children' in option) {
      return (
        <MultiSelectGroup key={option.value || index} value={option.value} heading={option.heading}>
          {renderMultiSelectOptions(option.children)}
        </MultiSelectGroup>
      )
    }

    return (
      <MultiSelectItem key={option.value} {...option}>
        {option.label}
      </MultiSelectItem>
    )
  })
}

const MultiSelect = {
  Root: MultiSelectRoot,
  Content: MultiSelectContent,
  Empty: MultiSelectEmpty,
  Group: MultiSelectGroup,
  Item: MultiSelectItem,
  List: MultiSelectList,
  Search: MultiSelectSearch,
  Separator: MultiSelectSeparator,
  Trigger: MultiSelectTrigger,
  Value: MultiSelectValue,
}

export { MultiSelect, renderMultiSelectOptions }
