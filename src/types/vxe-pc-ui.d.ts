// Type declaration for vxe-pc-ui components
declare module 'vxe-pc-ui/lib/pager' {
  import { DefineComponent } from 'vue';
  const VxePager: DefineComponent;
  export default VxePager;
}

declare module 'vxe-pc-ui' {
  import { DefineComponent } from 'vue';
  import type { Plugin } from 'vue';

  export const VxePager: DefineComponent;

  const defaultExport: Plugin;
  export default defaultExport;
}
