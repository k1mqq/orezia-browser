自作ブラウザ

なるべく小さい依存で作ってみたかった

コーディングにAIは使ってなくてすべて手書き

面白そうなところは作り終わってしまったのでやる気が出るまで更新しない

できること
 - URLパース
 - HTTPリクエスト
 - HTMLパース
 - softbuffer、winit、fontdueを使ったレンダリング（遅い）

できないこと
  - CSS、style属性でのスタイリング
  - テキスト以外のレンダリング
  - GET以外のリクエスト
  - 200以外のレスポンスコード対応
  - スクロールというか画面遷移すべて

依存
 - fontdue
 - rustls
 - softbuffer
 - webpki-roots
 - winit
