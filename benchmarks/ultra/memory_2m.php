<?php
ini_set('memory_limit', '2G');
class Entry {
    public int $id;
    public string $data;
    public function __construct(int $id, string $data) {
        $this->id = $id;
        $this->data = $data;
    }
}
function stress($count) {
    $m = array_fill(0, $count, null);
    for ($i = 0; $i < $count; $i++) {
        $m[$i] = new Entry($i, "some data");
    }
    return count($m);
}
$start = microtime(true);
$res = stress(2000000);
$end = microtime(true);
echo "PHP Memory Stress(2M) Result: $res\n";
echo "PHP Memory Stress(2M) Time: " . ($end - $start) . "s\n";
