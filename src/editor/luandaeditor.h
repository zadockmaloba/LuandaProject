#ifndef LUANDAEDITOR_H
#define LUANDAEDITOR_H

#include <QMainWindow>

class LuandaEditor : public QMainWindow
{
    Q_OBJECT
    struct LuandaEditorPrivate;
    LuandaEditorPrivate *_p = nullptr;

public:
    explicit LuandaEditor(QWidget *parent = nullptr);
    ~LuandaEditor() override;
    void addDockWidgetHelper(Qt::DockWidgetArea area, QWidget *widget);
};
#endif // LUANDAEDITOR_H
