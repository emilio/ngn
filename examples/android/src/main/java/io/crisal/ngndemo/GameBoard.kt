package io.crisal.ngndemo

// Pretty inspired by:
//   https://github.com/gabrielecirulli/2048/blob/478b6ec346e3787f589e4af751378d06ded4cbbc/js/game_manager.js

import android.util.Log
import kotlin.random.Random
import kotlinx.serialization.Serializable

enum class GameState {
    NotYetStarted, MyTurn, OtherTurn, Lost, Won,
}

enum class MoveDirection {
    Up, Down, Left, Right,
}

@Serializable
data class Point(val x: Int, val y: Int)

@Serializable
data class Tile(var value: Int, var pos: Point)

data class FurthestPosition(val pos: Point, val next: Point)

class GameBoard {
    val size = 4
    var state: GameState = GameState.NotYetStarted
        private set

    val board: Array<Array<Tile?>> = Array(size) { Array(size) { null } }

    val tiles = arrayListOf<Tile>()

    fun availableCells(): ArrayList<Point> {
        val result = arrayListOf<Point>()
        for (x in 0..<size) {
            for (y in 0..<size) {
                if (board[x][y] == null) {
                    result.add(Point(x, y))
                }
            }
        }
        return result
    }

    fun score(): Int {
        var score = 0
        for (tile in tiles) {
            score += tile.value
        }
        return score
    }

    fun posInBounds(pos: Point): Boolean {
        return pos.x >= 0 && pos.x < size && pos.y >= 0 && pos.y < size;
    }

    fun tileAt(pos: Point): Tile? {
        if (posInBounds(pos)) {
            return board[pos.x][pos.y]
        }
        return null
    }

    fun cellAvailable(pos: Point): Boolean {
        return posInBounds(pos) && tileAt(pos) == null
    }

    fun findFurthestPosition(cell: Point, direction: Point): FurthestPosition {
        var cell = cell
        var previous: Point

        // Progress towards the vector direction until an obstacle is found
        do {
            previous = cell
            cell = Point(previous.x + direction.x, previous.y + direction.y)
        } while (cellAvailable(cell))

        return FurthestPosition(previous, cell)
    }

    private fun addTile(value: Int, pos: Point): Tile {
        assert(board[pos.x][pos.y] == null) { "Tile already placed" }
        val tile = Tile(value, pos)
        board[pos.x][pos.y] = tile
        tiles.add(tile)
        return tile
    }

    private fun removeTile(tile: Tile) {
        assert(tiles.contains(tile))
        assert(tileAt(tile.pos) == tile)
        board[tile.pos.x][tile.pos.y] = null
        tiles.remove(tile)
    }

    private fun moveTile(tile: Tile, pos: Point) {
        assert(tiles.contains(tile))
        assert(tileAt(tile.pos) == tile)
        board[tile.pos.x][tile.pos.y] = null
        tile.pos = pos
        board[pos.x][pos.y] = tile
    }

    fun tryAddRandomTile(): Tile? {
        val value = if (Random.nextFloat() < .9) {
            2
        } else {
            4
        }
        return tryAddNewTile(value)
    }

    fun start() {
        if (state != GameState.NotYetStarted) {
            throw Error("Game is already started")
        }
        tryAddRandomTile()
        state = GameState.MyTurn
    }

    fun startWithTile(tile: Tile) {
        reset()
        addTile(tile.value, tile.pos)
        state = GameState.OtherTurn
    }

    fun tryAddNewTile(value: Int): Tile? {
        val cells = availableCells()
        if (cells.isEmpty()) {
            state = GameState.Lost
            return null
        }
        val pos = cells[Random.nextInt(until = cells.size)]
        return addTile(value, pos)
    }

    fun tryMove(direction: MoveDirection, tileToCreate: Tile? = null): Tile? {
        Log.d(TAG, "tryMove($direction)")
        if (state != GameState.MyTurn && state != GameState.OtherTurn) {
            Log.d(TAG, " > early out")
            return null
        }

        val (vector, xRange, yRange) = when (direction) {
            MoveDirection.Up -> Triple(Point(0, -1), 0 until size, 0 until size)
            MoveDirection.Down -> Triple(Point(0, 1), 0 until size, size - 1 downTo 0)
            MoveDirection.Left -> Triple(Point(-1, 0), 0 until size, 0 until size)
            MoveDirection.Right -> Triple(Point(1, 0), size - 1 downTo 0, 0 until size)
        }

        val merged = hashSetOf<Point>()
        var moved = false
        for (x in xRange) {
            for (y in yRange) {
                val pos = Point(x, y)
                val tile = tileAt(pos)
                if (tile == null) {
                    continue
                }
                val furthest = findFurthestPosition(pos, vector)
                val next = tileAt(furthest.next)
                if (next != null && next.value == tile.value && merged.add(furthest.next)) {
                    Log.d(TAG, " > merging $tile with $next")
                    next.value *= 2
                    // Remove the tile we merged into `next`.
                    removeTile(tile)
                    moved = true
                    if (next.value == 2048) {
                        state = GameState.Won
                    }
                } else {
                    if (pos != furthest.pos) {
                        Log.d(TAG, " > moving $tile to ${furthest.pos}")
                        moveTile(tile, furthest.pos)
                        moved = true // The tile moved from its original cell!
                    }
                }
            }
        }
        if (!moved) {
            return null
        }
        val tile = if (tileToCreate != null) {
            addTile(tileToCreate.value, tileToCreate.pos)
        } else {
            tryAddRandomTile()
        }
        state = if (tile == null) {
            GameState.Lost
        } else if (state == GameState.MyTurn) {
            GameState.OtherTurn
        } else {
            GameState.MyTurn
        }
        return tile
    }

    fun cellValues(): List<Int> {
        val result = arrayListOf<Int>()
        for (x in 0..<size) {
            for (y in 0..<size) {
                val tile = tileAt(Point(x, y))
                result.add(
                    tile?.value ?: 0
                )
            }
        }
        return result
    }

    fun reset() {
        for (tile in ArrayList(tiles)) {
            removeTile(tile)
        }
        state = GameState.NotYetStarted
    }
}
