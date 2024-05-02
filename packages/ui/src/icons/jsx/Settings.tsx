import * as React from 'react'
import type { SVGProps } from 'react'
const SvgSettings = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <g stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" clipPath="url(#Settings_svg__a)">
      <path d="M6.127 1.731A.875.875 0 0 1 6.991 1h2.017c.427 0 .793.31.863.731l.166.996c.049.291.243.534.501.677q.087.047.171.099c.253.152.56.2.837.096l.946-.354a.875.875 0 0 1 1.066.38l1.008 1.748a.875.875 0 0 1-.203 1.113l-.78.644a.96.96 0 0 0-.334.771V8.1a.96.96 0 0 0 .334.77l.781.644c.33.272.415.742.202 1.112l-1.01 1.747a.875.875 0 0 1-1.064.382l-.947-.354a.97.97 0 0 0-.836.096l-.171.1a.97.97 0 0 0-.501.676l-.166.996a.875.875 0 0 1-.863.731H6.99a.876.876 0 0 1-.863-.731l-.166-.996a.97.97 0 0 0-.5-.677l-.172-.099a.97.97 0 0 0-.837-.096l-.946.354a.875.875 0 0 1-1.065-.38l-1.009-1.748a.875.875 0 0 1 .202-1.113l.781-.644a.97.97 0 0 0 .335-.77V7.9a.97.97 0 0 0-.335-.771l-.78-.644a.875.875 0 0 1-.203-1.112l1.009-1.747a.875.875 0 0 1 1.065-.382l.946.354a.97.97 0 0 0 .837-.096q.085-.05.171-.1a.97.97 0 0 0 .501-.676z" />
      <path d="M10.5 8a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0" />
    </g>
    <defs>
      <clipPath id="Settings_svg__a">
        <path fill="#fff" d="M0 0h16v16H0z" />
      </clipPath>
    </defs>
  </svg>
)
export default SvgSettings
