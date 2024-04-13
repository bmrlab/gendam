'use client'
import classNames from 'classnames'
import { Toaster, toast } from 'sonner'

type ToasterProps = React.ComponentProps<typeof Toaster>

const SonnerToaster = ({ ...props }: ToasterProps) => {
  ;(window as any).toast = toast
  return (
    <Toaster
      // theme='dark' | 'light'
      // cn={classNames}
      closeButton={true}
      toastOptions={{
        unstyled: true,
        classNames: {
          toast: classNames(
            'group',
            'bg-app-box border border-app-line text-ink rounded-md shadow-xl pl-4 pr-6 py-4',
            'w-full flex items-center justify-start gap-2',
          ),
          // error: 'border border-red-400 bg-red-400 text-white',
          // error: 'text-red-400',
          // success: 'text-green-400',
          // warning: 'text-orange-400',
          // info: 'text-neutral-400',
          icon: classNames(
            'group-data-[type="error"]:text-red-500',
            'group-data-[type="success"]:text-green-500',
            'group-data-[type="warning"]:text-orange-500',
            'group-data-[type="info"]:text-neutral-500',
          ),
          content: 'flex-1 break-all',
          title: 'text-ink text-sm',
          description: 'text-ink/70 text-xs',
          actionButton: 'bg-app-hover text-xs rounded px-2 py-1',
          cancelButton: 'bg-app-hover text-xs rounded px-2 py-1',
          closeButton: classNames(
            'cursor-default transition-all duration-200',
            'top-1 right-1 p-1 rounded-none left-auto transform-none',
            'bg-transparent border-none text-ink/30',
            'group-hover:bg-transparent hover:text-ink/70',
            // 'bg-app-box text-ink border-app-line invisible opacity-0',
            // 'group-hover:bg-app-box group-hover:border-app-line group-hover:visible group-hover:opacity-100',
          ),
        },
      }}
      {...props}
    />
  )
}

export default SonnerToaster
