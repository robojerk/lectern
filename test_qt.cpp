#include <QApplication>
#include <QWidget>
#include <QLabel>
#include <QVBoxLayout>

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);

    QWidget window;
    window.setWindowTitle("Qt Test Window");
    window.resize(400, 300);

    QLabel *label = new QLabel("Qt GUI is WORKING! 🎉");
    label->setAlignment(Qt::AlignCenter);

    QVBoxLayout *layout = new QVBoxLayout;
    layout->addWidget(label);
    window.setLayout(layout);

    window.show();

    return app.exec();
}