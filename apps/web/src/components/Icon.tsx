import React from 'react'

export type SvgIconProps = React.SVGProps<SVGSVGElement>

export default {
  copy: (props: SvgIconProps) => (
    <svg {...props} width="16" height="16" viewBox="0 0 16 16" fill="transparent" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M13.5623 11.5397V2.43768H4.45996"
        stroke="white"
        strokeWidth="1.01132"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M11.54 4.46045H2.43774V13.5624H11.54V4.46045Z"
        stroke="white"
        strokeWidth="1.01132"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  arrowDown: (props: SvgIconProps) => (
    <svg {...props} width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
      <g>
        <path
          d="M3.66895 6.08838C3.66895 6.24951 3.72754 6.38135 3.83496 6.49365L7.5459 10.2925C7.68262 10.4243 7.82422 10.4927 8 10.4927C8.1709 10.4927 8.32227 10.4292 8.44922 10.2925L12.165 6.49365C12.2725 6.38135 12.3311 6.24951 12.3311 6.08838C12.3311 5.76123 12.0771 5.50732 11.7549 5.50732C11.5986 5.50732 11.4473 5.57568 11.335 5.68311L7.99512 9.10596L4.66504 5.68311C4.54785 5.5708 4.40625 5.50732 4.24512 5.50732C3.92285 5.50732 3.66895 5.76123 3.66895 6.08838Z"
          fill="currentColor"
        />
      </g>
    </svg>
  ),
  checked: (props: SvgIconProps) => (
    <svg {...props} width="16" height="17" viewBox="0 0 16 17" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M13.5 5.00037L6.5 12.0001L3 8.50037"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
}
