import type { Metadata, Viewport } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "人生 RPG 面板",
  description: "个人成长游戏化面板：属性六维 + 技能树 + 每日打卡 + 项目事件轴 + 集卡成就（明亮街机风）",
  // PWA：让 iPhone Safari「添加到主屏幕」后以独立应用形态启动（没有地址栏）
  manifest: "/manifest.webmanifest",
  applicationName: "人生 RPG 面板",
  appleWebApp: {
    capable: true,
    title: "人生面板",
    statusBarStyle: "default",
  },
  icons: {
    icon: [{ url: "/icon-192.png", sizes: "192x192", type: "image/png" }],
    // iOS 只认这张 180×180，且必须是不带透明圆角的整幅正方形
    apple: [{ url: "/apple-touch-icon.png", sizes: "180x180", type: "image/png" }],
  },
  formatDetection: { telephone: false },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  viewportFit: "cover",
  themeColor: "#F59E0B",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh-CN" className="h-full antialiased">
      <head>
        {/*
          iOS 独立模式开关。Next 15 的 metadata.appleWebApp 只生成新标准的
          `mobile-web-app-capable`，而 iPhone Safari 的「添加到主屏幕」在多数
          iOS 版本上仍只认带 Apple 前缀的这一个，所以两个都保留。
        */}
        <meta name="apple-mobile-web-app-capable" content="yes" />
        {/* 字体：jsdelivr @fontsource CDN（浏览器侧加载、失败自动回退系统字体） */}
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource/noto-sans-sc@5.1.0/400.css" />
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource/noto-sans-sc@5.1.0/700.css" />
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource/noto-sans-sc@5.1.0/900.css" />
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource/press-start-2p@5.1.0/400.css" />
      </head>
      <body>{children}</body>
    </html>
  );
}
