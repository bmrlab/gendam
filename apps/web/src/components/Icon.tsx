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
  more: (props: SvgIconProps) => (
    <svg {...props} width="17" height="16" viewBox="0 0 17 16" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M4.5625 8C4.5625 8.72487 3.97487 9.3125 3.25 9.3125C2.52513 9.3125 1.9375 8.72487 1.9375 8C1.9375 7.27513 2.52513 6.6875 3.25 6.6875C3.97487 6.6875 4.5625 7.27513 4.5625 8ZM9.8125 8C9.8125 8.72487 9.22487 9.3125 8.5 9.3125C7.77513 9.3125 7.1875 8.72487 7.1875 8C7.1875 7.27513 7.77513 6.6875 8.5 6.6875C9.22487 6.6875 9.8125 7.27513 9.8125 8ZM13.75 9.3125C14.4749 9.3125 15.0625 8.72487 15.0625 8C15.0625 7.27513 14.4749 6.6875 13.75 6.6875C13.0251 6.6875 12.4375 7.27513 12.4375 8C12.4375 8.72487 13.0251 9.3125 13.75 9.3125Z"
        fill="currentColor"
      />
    </svg>
  ),
  regenerate: (props: SvgIconProps) => (
    <svg {...props} width="16" height="17" viewBox="0 0 16 17" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M11.0105 6.73242H14.0105V3.73242" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
      <path
        d="M4.11084 4.61091C4.62156 4.10019 5.22788 3.69506 5.89517 3.41866C6.56246 3.14226 7.27766 3 7.99993 3C8.7222 3 9.4374 3.14226 10.1047 3.41866C10.772 3.69506 11.3783 4.10019 11.889 4.61091L14.0103 6.73223"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path d="M4.9895 10.2676H1.9895V13.2676" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
      <path
        d="M11.889 12.3891C11.3783 12.8999 10.772 13.305 10.1047 13.5814C9.43738 13.8578 8.72218 14.0001 7.99991 14.0001C7.27764 14.0001 6.56244 13.8578 5.89515 13.5814C5.22786 13.305 4.62154 12.8999 4.11082 12.3891L1.9895 10.2678"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  arrowUpLeft: (props: SvgIconProps) => (
    <svg width="16" height="17" viewBox="0 0 16 17" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M5 9L2 6L5 3" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
      <path
        d="M5 13H10.5C11.4283 13 12.3185 12.6313 12.9749 11.9749C13.6313 11.3185 14 10.4283 14 9.5V9.49999C14 9.04037 13.9095 8.58524 13.7336 8.1606C13.5577 7.73596 13.2999 7.35013 12.9749 7.02512C12.6499 6.70012 12.264 6.44231 11.8394 6.26642C11.4148 6.09053 10.9596 6 10.5 6H2"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  check: (props: SvgIconProps) => (
    <svg {...props} width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M11.8125 3.93784L5.6875 10.0626L2.625 7.00034"
        stroke="currentColor"
        strokeWidth="0.84375"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  error: (props: SvgIconProps) => (
    <svg {...props} width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M2.75314 2.75314C2.924 2.58229 3.201 2.58229 3.37186 2.75314L7 6.38128L10.6281 2.75314C10.799 2.58229 11.076 2.58229 11.2469 2.75314C11.4177 2.924 11.4177 3.201 11.2469 3.37186L7.61872 7L11.2469 10.6281C11.4177 10.799 11.4177 11.076 11.2469 11.2469C11.076 11.4177 10.799 11.4177 10.6281 11.2469L7 7.61872L3.37186 11.2469C3.201 11.4177 2.924 11.4177 2.75314 11.2469C2.58229 11.076 2.58229 10.799 2.75314 10.6281L6.38128 7L2.75314 3.37186C2.58229 3.201 2.58229 2.924 2.75314 2.75314Z"
        fill="currentColor"
      />
    </svg>
  ),
  loading: (props: SvgIconProps) => (
    <svg {...props} width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg">
      <g clipPath="url(#clip0_154_70728)">
        <path
          d="M13.125 7.49C12.8538 7.49 12.635 7.27125 12.635 7C12.635 6.18625 12.4775 5.39875 12.1625 4.655C11.8563 3.94625 11.4188 3.29 10.8675 2.7475C10.325 2.19625 9.66875 1.75875 8.96 1.4525C8.21625 1.14625 7.42875 0.98 6.615 0.98C6.34375 0.98 6.125 0.76125 6.125 0.49C6.125 0.21875 6.34375 0 6.615 0C7.56 0 8.47875 0.18375 9.33625 0.55125C10.1675 0.90125 10.92 1.40875 11.5588 2.0475C12.1975 2.68625 12.705 3.43875 13.055 4.27C13.4313 5.13625 13.615 6.055 13.615 7C13.615 7.27125 13.3962 7.49 13.125 7.49Z"
          fill="currentColor"
        />
      </g>
      <defs>
        <clipPath id="clip0_154_70728">
          <rect width="7.49" height="7.49" fill="white" transform="translate(6.125)" />
        </clipPath>
      </defs>
    </svg>
  ),
  moreVertical: (props: SvgIconProps) => (
    <svg {...props} width="25" height="25" viewBox="0 0 25 25" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M12.5 16.4375C13.2249 16.4375 13.8125 17.0251 13.8125 17.75C13.8125 18.4749 13.2249 19.0625 12.5 19.0625C11.7751 19.0625 11.1875 18.4749 11.1875 17.75C11.1875 17.0251 11.7751 16.4375 12.5 16.4375ZM12.5 11.1875C13.2249 11.1875 13.8125 11.7751 13.8125 12.5C13.8125 13.2249 13.2249 13.8125 12.5 13.8125C11.7751 13.8125 11.1875 13.2249 11.1875 12.5C11.1875 11.7751 11.7751 11.1875 12.5 11.1875ZM13.8125 7.25C13.8125 6.52513 13.2249 5.9375 12.5 5.9375C11.7751 5.9375 11.1875 6.52513 11.1875 7.25C11.1875 7.97487 11.7751 8.5625 12.5 8.5625C13.2249 8.5625 13.8125 7.97487 13.8125 7.25Z"
        fill="currentColor"
      />
    </svg>
  ),
  circleX: (props: SvgIconProps) => (
    <svg width="25" height="25" viewBox="0 0 25 25" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M12.5 18.5C15.8137 18.5 18.5 15.8137 18.5 12.5C18.5 9.18629 15.8137 6.5 12.5 6.5C9.18629 6.5 6.5 9.18629 6.5 12.5C6.5 15.8137 9.18629 18.5 12.5 18.5Z"
        stroke="#676C77"
        strokeWidth="0.9"
        strokeMiterlimit="10"
      />
      <path
        d="M14.5 10.5L10.5 14.5"
        stroke="currentColor"
        strokeWidth="0.9"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M14.5 14.5L10.5 10.5"
        stroke="currentColor"
        strokeWidth="0.9"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  trash: (props: SvgIconProps) => (
    <svg width="16" height="17" viewBox="0 0 16 17" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M10.8645 14.6538H5.1361C4.14729 14.6538 3.34283 13.8494 3.34283 12.8606V4.41486H3.99157V12.8606C3.99157 13.4917 4.50498 14.0051 5.1361 14.0051H10.8645C11.4956 14.0051 12.009 13.4917 12.009 12.8606V4.41486H12.6577V12.8606C12.6577 13.8494 11.8533 14.6538 10.8645 14.6538Z"
        fill="#currentColor"
        stroke="currentColor"
        strokeWidth="0.292685"
      />
      <path
        d="M13.8386 4.73894H2.1614C1.98224 4.73894 1.83704 4.59374 1.83704 4.41458C1.83704 4.23541 1.98224 4.09021 2.1614 4.09021H13.8386C14.0178 4.09021 14.163 4.23541 14.163 4.41458C14.163 4.59374 14.0178 4.73894 13.8386 4.73894Z"
        fill="currentColor"
        stroke="currentColor"
        strokeWidth="0.292685"
      />
      <path
        d="M10.206 4.41479H9.55726V3.40494C9.55726 3.17884 9.37328 2.99492 9.14724 2.99492H6.85209C6.62605 2.99492 6.44207 3.17884 6.44207 3.40494V4.41479H5.79333V3.40494C5.79333 2.82115 6.26823 2.34619 6.85209 2.34619H9.14724C9.7311 2.34619 10.206 2.82115 10.206 3.40494V4.41479Z"
        fill="currentColor"
        stroke="currentColor"
        strokeWidth="0.292685"
      />
      <path
        d="M6.28012 12.1668C6.10095 12.1668 5.95575 12.0216 5.95575 11.8425V7.08181C5.95575 6.90265 6.10095 6.75745 6.28012 6.75745C6.45928 6.75745 6.60448 6.90265 6.60448 7.08181V11.8425C6.60448 12.0216 6.45928 12.1668 6.28012 12.1668Z"
        fill="currentColor"
        stroke="currentColor"
        strokeWidth="0.292685"
      />
      <path
        d="M9.71908 12.1668C9.53992 12.1668 9.39471 12.0216 9.39471 11.8425V7.08181C9.39471 6.90265 9.53992 6.75745 9.71908 6.75745C9.89824 6.75745 10.0434 6.90265 10.0434 7.08181V11.8425C10.0434 12.0216 9.89824 12.1668 9.71908 12.1668Z"
        fill="currentColor"
        stroke="currentColor"
        strokeWidth="0.292685"
      />
    </svg>
  ),
  cancel: (props: SvgIconProps) => (
    <svg width="16" height="17" viewBox="0 0 16 17" fill="transparent" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M8 14.5C11.3137 14.5 14 11.8137 14 8.5C14 5.18629 11.3137 2.5 8 2.5C4.68629 2.5 2 5.18629 2 8.5C2 11.8137 4.68629 14.5 8 14.5Z"
        stroke="currentColor"
        strokeWidth="0.9"
        strokeMiterlimit="10"
      />
      <path d="M10 6.5L6 10.5" stroke="currentColor" strokeWidth="0.9" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M10 10.5L6 6.5" stroke="currentColor" strokeWidth="0.9" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  ),
  download: (props: SvgIconProps) => (
    <svg width="16" height="17" viewBox="0 0 16 17" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M5.375 7.375L8 10L10.625 7.375"
        stroke="currentColor"
        strokeOpacity="0.95"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M8 2.99817V9.99817"
        stroke="currentColor"
        strokeOpacity="0.95"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M14 9V13.5C14 13.6326 13.9473 13.7598 13.8536 13.8536C13.7598 13.9473 13.6326 14 13.5 14H2.5C2.36739 14 2.24021 13.9473 2.14645 13.8536C2.05268 13.7598 2 13.6326 2 13.5V9"
        stroke="currentColor"
        strokeOpacity="0.95"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  ),
  audio: (props: SvgIconProps) => (
    <svg width="16" height="17" viewBox="0 0 16 17" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M5 11H2C1.86739 11 1.74021 10.9473 1.64645 10.8536C1.55268 10.7598 1.5 10.6326 1.5 10.5V6.5C1.5 6.36739 1.55268 6.24021 1.64645 6.14645C1.74021 6.05268 1.86739 6 2 6H5L9.5 2.5V14.5L5 11Z"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path d="M15 7L12 10" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M15 10L12 7" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  ),
}
