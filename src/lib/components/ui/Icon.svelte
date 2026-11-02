<script lang="ts" module>
  // 2px-stroke line icons, taken from the design mockups.
  const paths = {
    hash: ["M4 9h16M4 15h16M10 3 8 21M16 3l-2 18"],
    volume: ["M11 5 6 9H2v6h4l5 4z", "M15.5 8.5a5 5 0 0 1 0 7", "M19 5a10 10 0 0 1 0 14"],
    servers: ["M3 6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM3 15a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z", "M7 7.5h.01M7 16.5h.01"],
    users: ["M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2", "M9 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8z", "M22 21v-2a4 4 0 0 0-3-3.9M16 3.1a4 4 0 0 1 0 7.8"],
    edit: ["M12 20h9", "M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"],
    trash: ["M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6"],
    send: ["m22 2-7 20-4-9-9-4z", "M22 2 11 13"],
    logout: ["M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4", "M16 17l5-5-5-5", "M21 12H9"],
    close: ["M18 6 6 18M6 6l12 12"],
    alert: ["M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z", "M12 8v4M12 16h.01"],
    retry: ["M3 12a9 9 0 0 1 15.5-6.2L21 8", "M21 3v5h-5", "M21 12a9 9 0 0 1-15.5 6.2L3 16", "M3 21v-5h5"],
    arrowDown: ["M12 5v14", "M19 12l-7 7-7-7"],
  } as const;

  export type IconName = keyof typeof paths;
</script>

<script lang="ts">
  interface Props {
    name: IconName;
    size?: number;
    strokeWidth?: number;
    label?: string;
  }

  let { name, size = 18, strokeWidth = 2, label }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  role={label ? "img" : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : "true"}
>
  {#each paths[name] as d (d)}
    <path {d} />
  {/each}
</svg>

<style>
  svg {
    flex: 0 0 auto;
    display: block;
  }
</style>
