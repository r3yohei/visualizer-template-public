# AHC Visualizer
## 環境構築
### ubuntu
- yarn
```
apt-get install -y nodejs npm
npm install -g n
n lts
apt purge -y nodejs npm
apt autoremove -y
npm install -g yarn
```

- wasm
```
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

- rust
    - 1.70.0と念の為stableもinstall

## 開発手順
### 事前準備
1. 本リポジトリをbare cloneする
```
git clone --bare https://github.com/r3yohei/visualizer-template-public.git
```
2. githubで好きな名前で新しくリポジトリを作成
3. bareリポジトリを上記リポジトリへmirror pushする
```
cd visualizer-template-public.git/
# 例えば新しくvisualizer-ahc042というリポジトリを作った場合
git push --mirror https://github.com/r3yohei/visualizer-ahc042.git
```
4. bareリポジトリを削除し、上記リポジトリをcloneする
```
cd ..
rm -rf visualizer-template-public.git/
git clone https://github.com/r3yohei/visualizer-ahc042.git
```
5. middleware.jsの `user`と`password` をチームで決めたものに変更する
    - basic認証用です
6. nodeモジュールのインストール
```
# project rootにて
yarn
```
7. vercelでBASIC認証をつけられるようにする
```
yarn add @vercel/edge
```
### 本番中の開発
1. 配布されたローカルテスタのlib.rsと問題文を使っていったんLLMさんに聞いてみる
<details>

<summary>プロンプト</summary>
あなたにAtCoder Heuristic Contestのビジュアライザ・入力ジェネレーターの作成をお願いしたいです。
システムはReact + Rustによるwasmで構成されていて、概ね以下のような担当分けになっています:
React側: seed値・outputをtextareaから受け付けて、Rustに送る・Rustから受け取った入力ファイルをTextAreaに表示・Rustから受け取ったsvgを表示
Rust側: Reactから渡されたものに対して処理を行う: 
具体的には、
- seedの値に基づいて入力ファイルの作成
- 与えられた出力に基づいてビジュアライザの作成(svgの描画)、ターンごと
- 入力・出力を受け取って、最大のターン数を返す
ことを行なっています。
以下のコードはRust側の例で、インターフェースを変えずに(つまり、lib.rsの内容をほぼ変えずに)、別のコンテスト用のビジュアライザシステムの開発を行いたいです:

[lib.rs]
use wasm_bindgen::prelude::*;
mod util;

#[wasm_bindgen]
pub fn gen(seed: i32) -> String {
    util::gen(seed as u64).to_string()
}

#[wasm_bindgen(getter_with_clone)]
pub struct Ret {
    pub score: i64,
    pub err: String,
    pub svg: String,
}

#[wasm_bindgen]
pub fn vis(_input: String, _output: String, turn: usize) -> Ret {
    let input = util::parse_input(&_input);
    let output = util::parse_output(&_output);
    let (score, err, svg) = util::vis(&input, &output, turn);
    Ret {
        score: score as i64,
        err,
        svg,
    }
}

#[wasm_bindgen]
pub fn get_max_turn(_input: String, _output: String) -> usize {
    let output = util::parse_output(&_output);
    output.q
}

[util.rs]
#![allow(non_snake_case, unused_macros)]
use proconio::input;
use rand::prelude::*;
use std::collections::VecDeque;
use svg::node::element::{Rectangle, Style};
use web_sys::console::log_1;

pub trait SetMinMax {
    fn setmin(&mut self, v: Self) -> bool;
    fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn setmin(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn setmax(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

#[derive(Clone, Debug)]
pub struct Input {
    pub id: usize,
    pub n: usize,
    pub k: usize,
    pub s: Vec<String>,
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {} {}", self.id, self.n, self.k)?;
        for i in 0..self.n {
            writeln!(f, "{}", self.s[i])?;
        }
        Ok(())
    }
}

pub fn parse_input(f: &str) -> Input {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        id:usize,
        n: usize,
        k: usize,
        s: [String; n]
    }
    Input { id, n, k, s }
}

pub struct Output {
    pub q: usize,
    pub yxc: Vec<(usize, usize, usize)>,
}

pub fn parse_output(f: &str) -> Output {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        q: usize,
        yxc: [(usize, usize, usize); q]
    }
    Output { q, yxc }
}

pub fn gen(seed: u64) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    let id = seed;
    let n = 100;
    let k = 9;
    let s = (0..n)
        .map(|_| {
            (0..n)
                .map(|_| rng.gen_range(1..k + 1).to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    Input { id: 0, n, k, s }
}

fn calculate_score(input: &Input, yxc: &Vec<(usize, usize, usize)>) -> (usize, Vec<Vec<usize>>) {
    let mut state = vec![vec![0; input.n]; input.n];
    input.s.iter().enumerate().for_each(|(y, s)| {
        s.chars()
            .enumerate()
            .for_each(|(x, c)| state[y][x] = c.to_digit(10).unwrap() as usize)
    });

    let x_vec: Vec<i32> = vec![0, 1, 0, -1];
    let y_vec: Vec<i32> = vec![-1, 0, 1, 0];

    for (y, x, c) in yxc {
        // state[*y][*x] = *c;
        let selected_color = state[*y - 1][*x - 1];

        let mut visited = vec![vec![false; input.n]; input.n];
        let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
        queue.push_back((*y - 1, *x - 1));

        let mut count = 0;

        while queue.len() > 0 {
            let (ypos, xpos) = queue.pop_front().unwrap();
            if visited[ypos][xpos] {
                continue;
            }
            visited[ypos][xpos] = true;
            state[ypos][xpos] = *c;

            count = count + 1;
            for i in 0..4 {
                let nx = xpos as i32 + x_vec[i];
                let ny = ypos as i32 + y_vec[i];
                if nx < 0 || ny < 0 || nx >= input.n as i32 || ny >= input.n as i32 {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if visited[ny][nx] {
                    continue;
                }

                if state[ny][nx] != selected_color {
                    continue;
                }
                queue.push_back((ny, nx));
            }
        }
    }

    let mut score = 0;
    for color in 1..(input.k + 1) {
        let mut tmp_score = 0;
        for y in 0..input.n {
            for x in 0..input.n {
                if state[y][x] == color {
                    tmp_score += 100;
                }
            }
        }
        score = score.max(tmp_score);
    }
    score -= yxc.len();

    return (score, state);
}

fn generate_dark_color(code: usize) -> String {
    // 入力値に基づいてHue（色相）を計算
    let hue = (code as f32 * 36.0) % 360.0;

    // Saturation（彩度）を低めに、Lightness（明度）を固定値で低く設定
    let saturation = 30.0;
    let lightness = 30.0;

    // HSL to RGB 変換
    let hue_normalized = hue / 360.0;
    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - (lightness * saturation)
    };

    let p = 2.0 * lightness - q;

    let r = hue_to_rgb(p, q, hue_normalized + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, hue_normalized);
    let b = hue_to_rgb(p, q, hue_normalized - 1.0 / 3.0);

    // RGB を 16 進数に変換して文字列を返す
    format!(
        "#{:02X}{:02X}{:02X}",
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8
    )
}

fn generate_color(code: usize) -> String {
    // 入力値に基づいてHue（色相）を計算
    let hue = (code as f32 * 36.0) % 360.0;

    // Saturation（彩度）とLightness（明度）を固定値で設定
    let saturation = 10.0;
    let lightness = 0.1;

    // HSL to RGB 変換
    let hue_normalized = hue / 360.0;
    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - (lightness * saturation)
    };

    let p = 2.0 * lightness - q;

    let r = hue_to_rgb(p, q, hue_normalized + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, hue_normalized);
    let b = hue_to_rgb(p, q, hue_normalized - 1.0 / 3.0);

    // RGB を 16 進数に変換して文字列を返す
    format!(
        "#{:02X}{:02X}{:02X}",
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8
    )
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let t = if t < 0.0 {
        t + 1.0
    } else if t > 1.0 {
        t - 1.0
    } else {
        t
    };

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

pub fn rect(x: usize, y: usize, w: usize, h: usize, fill: &str) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", w)
        .set("height", h)
        .set("fill", fill)
}

pub fn vis(input: &Input, output: &Output, turn: usize) -> (i64, String, String) {
    let (score, state) =
        calculate_score(input, &output.yxc[0..turn].into_iter().cloned().collect());

    let W = 800;
    let H = 800;
    let w = 8;
    let h = 8;
    let mut doc = svg::Document::new()
        .set("id", "vis")
        .set("viewBox", (-5, -5, W + 10, H + 10))
        .set("width", W + 10)
        .set("height", H + 10)
        .set("style", "background-color:white");

    doc = doc.add(Style::new(format!(
        "text {{text-anchor: middle;dominant-baseline: central; font-size: {}}}",
        6
    )));
    for y in 0..input.n {
        for x in 0..input.n {
            doc = doc.add(
                rect(
                    x * w,
                    W - (y + 1) * h,
                    w,
                    h,
                    &generate_dark_color(state[y][x]),
                )
                .set("stroke", "black")
                .set("stroke-width", 1)
                .set("class", "box"),
            );
        }
    }

    (score as i64, "".to_string(), doc.to_string())
}


上記の情報を参考にして、この次に与えるAtCoder Heuristic Contestの問題のビジュアライザのためのutil.rsを書いてください。
ただし、元々のutil.rsの構造を大きく変えないで欲しいです:
- Input, Output構造体を作る
- Input,Outputに実装したトレイトは必ず実装する(特にDisplayを忘れがち)
- 適切にコメントを入れる
- 入力生成方法は簡易化せずに厳密に指定に従う必要があります
- 同じlib.rsを使うので、util.rsのインターフェースを変えることは禁止
- Rustのクレートは以下のバージョンのものを使用する:
wasm-bindgen = "0.2.89"
getrandom = {version="0.2", features=["js"]}
rand = { version = "=0.8.5", features = ["small_rng", "min_const_gen"] }
rand_chacha = "=0.3.1"
rand_distr = "=0.4.3"
itertools = "=0.11.0"
proconio = { version = "=0.4.5", features = ["derive"] }
clap = { version = "4.0.22", features = ["derive"] }
svg = "0.17.0"
delaunator = "1.0.1"
web-sys = {"version" = "0.3.44", features=['console']}

ただし、以下のコードを踏襲してInput, Output, genなどを書いてください。

[ツール類]
公式から配布されるtools/src/lib.rsをコピペする

[問題文]
AtCoderのサイトからコピペ (右クリック -> ページのソースを表示がいいかも？)

[ビジュアライザの仕様]
問題ごとにこのようにビジュアライザを作って欲しいという仕様を書く

</details>

2. wasmをbuild
```
cd wasm
wasm-pack build --target web --out-dir ../public/wasm
```
3. (初回のみ) public/wasmを.gitignoreから外す
4. ローカルで動確
```
yarn dev
```
5. vercelでホスティングする
    1. add new projectで開発したリポジトリを選ぶ
    2. build & development settingsでviteを選択し、build commandをtsc & vite buildとして実行