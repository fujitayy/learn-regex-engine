# メモ

- 正規表現のパース処理についてフォーカスするか？
  - オートマトンとそれによる受理の仕組みがメインなので、ここは端折ってもいい。
  - 余裕があったらやれば良いので、まずはnomとかでさくっとパースする前提で良い。
- パースして作成したオートマトンを使ってマッチング処理を実装する
  - ここがメイン
- 正規表現の実装方法は何を採用するか？
  - 分かりやすさ重視でいくのでNFAで実装する。
  - 状態遷移は深さ優先探索で実装し、最長マッチから最短マッチの順にバックトラックさせる。
    - 学習目的としてはコレで十分だと考える。
  - 問題は状態の保持と遷移の表現方法
    - 正規表現で構築されたオートマトンは他のステートからまた自分に返ってくるという2つ以上の状態を含む閉路を持たないDAGにできる。
    - さらに選択や繰り返しといった分岐先が複数になるケースはある状態の内部状態とする事ができる。
    - このようにすると基本的には左から右へ状態遷移が進むだけとなり、非常にシンプルに記述することができる。
    - オートマトンは正規表現のASTとほぼ等価な形で表現できる。
    - 次の状態は常に一意に定まるので比較的シンプルに構築できる。
    - この実装においてε遷移は*と選択の処理で現れる。
      - 内部状態への遷移がε遷移に相当する。
- 勉強会中で解説する事
  - 正規表現エンジンの基本動作原理
    - オートマトン理論
      - NFA
      - DFA
  - NFAベースのregex engineの実装方法
- 今回実装する正規表現の仕様
  - **解説する事項を可能な限り減らすのが最大の目的**
  - エスケープ無し (つまり . と * と | を含む文字列にマッチする正規表現は書けない)
    - 単純なアルファベット列を想定
  - 連接、ドット、繰り返し、選択、グループ化の4つをサポートする。
    - 連接: `aiueo`
    - ドット: `a.b`
    - 繰り返し: `a*`, `(ab*c)*`
    - 選択: `a|b`
    - グループ化: `(a)`, `(a|b)`, `(a|b)|(c|d)`

## `e*`のパースについて

`正規表現全体 + maybe *` と考える

連接、ドット、繰り返しをサポートする正規表現であればその文法はBNF風に書くと以下のようになる

regex ::= partial { partial }

partial ::= word
          | repeat
          | grouped

repeat ::= word "*"
         | grouped "*"

grouped ::= "(" groupable { groupable } ")"

groupable ::= word
            | repeat

word ::= char { char }

char ::= [a-zA-Z0-9]
       | "."

## 参考にした情報

- 正規表現エンジンメモ <https://jetbead.hatenablog.com/entry/20120917/1347825073>