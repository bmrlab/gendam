import * as React from 'react'

export type SvgIconProps = React.SVGProps<SVGSVGElement>

import * as Pack1 from './pack1'
import * as Pack2 from './pack2'
import * as Pack3 from './pack3'

export default {
  ...Pack1,
  ...Pack2,
  ...Pack3,
}
