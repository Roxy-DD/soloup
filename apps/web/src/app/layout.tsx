import type { Metadata, Viewport } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "人生 RPG 面板",
  description: "个人成长游戏化面板：属性六维 + 技能树 + 每日打卡 + 项目事件轴 + 集卡成就（明亮街机风）",
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  viewportFit: "cover",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh-CN" className="h-full antialiased">
      <head>
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
