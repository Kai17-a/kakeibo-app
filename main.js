// Start Express
const expressApp = require('./app');
expressApp.listen(3000, '127.0.0.1');

// Electronのモジュール
const electron = require('electron');

// アプリケーションをコントロールするモジュール
const app = electron.app;

// ウィンドウを作成するモジュール
const BrowserWindow = electron.BrowserWindow;

// メインウィンドウはGCされないようにグローバル宣言
let mainWindow;

// 全てのウィンドウが閉じたら終了
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
})

// Electronの初期化完了後に実行
app.on('ready', () => {
  // メイン画面の表示。ウィンドウの幅、高さを指定できる
  mainWindow = new BrowserWindow({
    height: 810,
    width: 1440, 
    maxHeight:1080,
    maxWidth: 1920,
    minHeight: 540,
    minWidth: 960
  });
  mainWindow.loadURL('http://127.0.0.1:3000');
  mainWindow.setMenu(null);

  // ウィンドウが閉じられたらアプリも終了
  mainWindow.on('closed', () => {
    mainWindow = null;
  })
})