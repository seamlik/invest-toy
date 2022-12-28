mod ranker;

fn main() {
    ranker::StockRanker::default().rank(&Default::default());
}
