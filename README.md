# RusticNest Compiler

## 简介

RNC 是一个用 Rust 实现的某妙妙语言编译器。

## 功能

- 词法分析模块（使用 `Logos`）
- 语法分析模块（使用 `LALRPOP`）
- 语义分析模块
- 目标代码生成（目前仅能生成汇编代码）

## 支持的目标架构

- `loongarch32` with `ilp32s` ABI
