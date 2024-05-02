'use client'
import Icon from '@gendam/ui/icons'

type IconName = keyof typeof Icon;

const iconNames: IconName[] = [
  'Add',
  'ArrowClockwise',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'ArrowUp',
  'ArrowUpLeft',
  'Briefcase',
  'BulletList',
  'Check',
  'Clock',
  'Close',
  'CloseRounded',
  'Column',
  'Compass',
  'CopySimple',
  'Cycle',
  'Download',
  'EditPen',
  'EditSquare',
  'File',
  'Finder',
  'FolderAdd',
  'FolderOpen',
  'Gallery',
  'Gear',
  'Grid',
  'Image',
  'Library',
  'List',
  'Loading',
  'MagnifyingGlass',
  'Mic',
  'Minus',
  'Model',
  'Moon',
  'MoreHorizontal',
  'MoreVertical',
  'Pinterest',
  'Play',
  'QuestionMark',
  'RemoveFromList',
  'RemoveSquare',
  'Rocket',
  'ScreenExpand',
  'ScreenNarrow',
  'SelfAdapting',
  'Settings',
  'Sidebar',
  'SkipBack',
  'SkipForward',
  'SpeakerHigh',
  'SpeakerSimpleX',
  'Sun',
  'Tag',
  'Trash',
  'UpAndDownArrow',
  'Upload',
]

export default function Icons() {
  return (
    <div className="flex flex-wrap gap-4 text-red-500">
      {iconNames.map(iconName => {
        const Component: any = Icon[iconName]
        return (
          <div className="w-32 text-center" key={iconName}>
            <div className="text-xs">{iconName}</div>
            <Component className="inline-block h-4 w-4" />
          </div>
        )
      })}
    </div>
  )
}
